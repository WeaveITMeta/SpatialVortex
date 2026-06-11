//! Rasmussen S/R/K dispatch — a learned router over the policy ladder.
//!
//! The agent has three qualitatively different ways to act, mirroring
//! Rasmussen's skill/rule/knowledge hierarchy:
//!
//!   Skill     — cached, automatized behavior (win recipes, greedy
//!               Q-values, the click-targets option). Cheap and right
//!               WHEN the mechanic is already understood.
//!   Rule      — if-then dispatch over verified laws in the epistemic
//!               ledger. Right when the situation matches a known claim
//!               but no automatized routine exists yet.
//!   Knowledge — deliberate model-building: exploration, hypothesis
//!               testing, predicate refinement. Expensive, but the only
//!               level that works in genuinely novel territory.
//!
//! The failure mode this module exists to prevent is MISDISPATCH:
//! grinding a stale recipe against a level that changed the rules
//! (skill where knowledge was needed), or burning hundreds of steps
//! re-deriving a mechanic the ledger already wrote as Law (knowledge
//! where skill was needed). Hand-tuned thresholds for "when is the world
//! novel enough to think?" kept rotting as games varied, so the
//! threshold is LEARNED instead: a linear softmax over a standardized
//! delta-feature stream, trained online as a gradient bandit from the
//! same shaped reward the rest of the policy uses. Linear-with-features
//! is deliberate — three classes and ~a dozen features mean every weight
//! is inspectable, the whole router serializes into the knowledge
//! snapshot in a few hundred bytes, and one bad episode cannot destroy
//! it the way it could a deep net (the 2GB-knowledge-file lesson again:
//! small, bounded, earned).

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Number of router features. One bias term plus the twelve standardized
/// delta signals in [`RouterInputs`]. Bump this only together with
/// [`DeltaFeatures::extract`]; deserialized routers from older snapshots
/// tolerate a smaller dimension (missing weights read as zero).
pub const FEATURE_DIM: usize = 13;

/// Raw, un-normalized signals the world model already computes each step.
/// This is the standardized contract between the world model and the
/// router: counts stay counts and ratios stay ratios HERE, and all
/// squashing/clamping happens in [`DeltaFeatures::extract`], so the world
/// model never needs to know how features are encoded.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouterInputs {
    /// Grid cell-changes this step net of ambient dynamics (changes the
    /// environment makes on its own do not signal our action mattered).
    pub n_changes_net_ambient: u32,
    /// Object-level events (appear/vanish/move/recolor) from the tracker.
    pub n_object_events: u32,
    /// Global representation accuracy — how well the forward model has
    /// been predicting lately, in [0,1].
    pub global_ira: f32,
    /// Recent slope of IRA. Falling IRA is the single strongest "your
    /// model is going stale" signal — typically in [-1,1].
    pub ira_trend: f32,
    /// Win-predicate ensemble progress toward the believed goal, [0,1].
    pub predicate_progress: f32,
    /// Change in predicate progress since last step — typically [-1,1].
    pub predicate_progress_delta: f32,
    /// Has any control mechanic been verified (ledger has relevant Laws)?
    pub mechanic_known: bool,
    /// How many action→effect mappings are known (size of control map).
    pub control_map_size: u32,
    /// Remaining target objects/cells per the current interpretation.
    pub targets_remaining: u32,
    /// Is the current abstract state one we have never seen before?
    pub novel_state: bool,
    /// Did the last movement/action get blocked (no effect where one was
    /// predicted)?
    pub blocked: bool,
    /// Steps since the last level advance — staleness pressure.
    pub steps_since_level_advance: u32,
}

/// Stateless featurizer from [`RouterInputs`] to the fixed-size vector
/// the linear router consumes.
///
/// Encoding rules (the WHY of each transform):
/// - Unbounded counts go through x/(1+x): the difference between 0 and 3
///   changes matters far more than between 40 and 43, and a bounded
///   feature keeps any single step from swinging weights violently.
/// - Ratios are clamped to their nominal range so a buggy upstream value
///   cannot blow up the softmax.
/// - Booleans become 0/1.
/// - Feature 0 is a constant bias so each class can carry a context-free
///   prior logit.
pub struct DeltaFeatures;

impl DeltaFeatures {
    /// Build the feature vector. Pure and total: any `RouterInputs` maps
    /// to features within [-1, 1].
    pub fn extract(inp: &RouterInputs) -> [f32; FEATURE_DIM] {
        fn squash(x: u32) -> f32 {
            let x = x as f32;
            x / (1.0 + x)
        }
        fn b(x: bool) -> f32 {
            if x {
                1.0
            } else {
                0.0
            }
        }
        [
            1.0, // bias
            squash(inp.n_changes_net_ambient),
            squash(inp.n_object_events),
            inp.global_ira.clamp(0.0, 1.0),
            inp.ira_trend.clamp(-1.0, 1.0),
            inp.predicate_progress.clamp(0.0, 1.0),
            inp.predicate_progress_delta.clamp(-1.0, 1.0),
            b(inp.mechanic_known),
            squash(inp.control_map_size),
            squash(inp.targets_remaining),
            b(inp.novel_state),
            b(inp.blocked),
            squash(inp.steps_since_level_advance),
        ]
    }
}

/// Which level of the policy ladder to trust this step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DispatchClass {
    /// Execute cached behavior: recipes, greedy Q, known click targets.
    Skill,
    /// Dispatch on verified laws: match situation → apply known rule.
    Rule,
    /// Deliberate: explore, test hypotheses, rebuild the model.
    Knowledge,
}

impl DispatchClass {
    /// All classes in weight-row order. The order is part of the
    /// serialization contract — do not reorder.
    pub const ALL: [DispatchClass; 3] =
        [DispatchClass::Skill, DispatchClass::Rule, DispatchClass::Knowledge];

    /// Row index into the router's weight matrix.
    pub fn index(&self) -> usize {
        match self {
            DispatchClass::Skill => 0,
            DispatchClass::Rule => 1,
            DispatchClass::Knowledge => 2,
        }
    }
}

/// Linear softmax router trained as a gradient bandit.
///
/// Three logits (one per [`DispatchClass`]), each a dot product of a
/// weight row with the delta features; softmax turns logits into a
/// dispatch distribution. Updates follow the gradient-bandit rule with an
/// EMA reward baseline: weights move proportionally to the ADVANTAGE
/// (reward minus baseline), pulled toward the chosen class and pushed
/// away from the others, scaled by the features that were active — so
/// credit lands on the context, not just the class.
///
/// Weights are stored as `Vec<Vec<f32>>` rather than `[[f32; N]; 3]`
/// purely for painless serde (fixed-size float arrays need manual impls);
/// shape is re-asserted on every update so a snapshot from an older,
/// smaller `FEATURE_DIM` loads cleanly with zeros for the new features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearRouter {
    /// weights[class.index()][feature].
    weights: Vec<Vec<f32>>,
    /// EMA of shaped reward — the gradient-bandit baseline. Without it,
    /// any uniformly positive reward stream would inflate ALL logits and
    /// learning would stall; with it, only better-than-usual outcomes
    /// move weights.
    baseline: f32,
    /// Step size for weight updates.
    lr: f32,
    /// EMA rate for the baseline.
    baseline_rate: f32,
    /// Total updates applied — telemetry, and a cheap "has this router
    /// actually learned anything yet" signal for callers.
    updates: u64,
}

impl LinearRouter {
    /// Fresh router with a tiny DETERMINISTIC tie-break init.
    ///
    /// No RNG: replays must be reproducible, and the init must not smuggle
    /// in semantic priors (e.g. "Knowledge when novel") — those are
    /// exactly what the bandit is supposed to EARN from reward, and a
    /// baked-in prior would mask a broken update rule in testing. The
    /// offsets only ensure argmax is well-defined before any learning.
    pub fn new() -> Self {
        let mut weights = vec![vec![0.0f32; FEATURE_DIM]; 3];
        for (k, row) in weights.iter_mut().enumerate() {
            for (i, w) in row.iter_mut().enumerate() {
                // Structured, tiny, deterministic: in [-3e-3, 3e-3].
                *w = 1e-3 * ((((k * 31 + i * 17) % 7) as f32) - 3.0);
            }
        }
        Self {
            weights,
            baseline: 0.0,
            lr: 0.15,
            baseline_rate: 0.05,
            updates: 0,
        }
    }

    /// Softmax dispatch distribution over [Skill, Rule, Knowledge] for
    /// the given features. Exposed for telemetry — the decision log wants
    /// the full distribution, not just the winner.
    pub fn probs(&self, f: &[f32; FEATURE_DIM]) -> [f32; 3] {
        let mut z = [0.0f32; 3];
        for (k, zk) in z.iter_mut().enumerate() {
            *zk = (0..FEATURE_DIM)
                .map(|i| {
                    self.weights
                        .get(k)
                        .and_then(|row| row.get(i))
                        .copied()
                        .unwrap_or(0.0)
                        * f[i]
                })
                .sum();
        }
        // Max-subtraction for numerical stability.
        let m = z.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let mut e = [0.0f32; 3];
        let mut sum = 0.0f32;
        for (k, ek) in e.iter_mut().enumerate() {
            *ek = (z[k] - m).exp();
            sum += *ek;
        }
        [e[0] / sum, e[1] / sum, e[2] / sum]
    }

    /// Pick the dispatch class. Greedy argmax over the softmax — the
    /// policy components underneath carry their own exploration, so the
    /// router itself stays deterministic (and replayable). Returns the
    /// class and its softmax probability as a confidence in [0,1]; a
    /// confidence near 1/3 means "the router has no opinion", which
    /// callers may treat as a cue to fall back to their own default.
    pub fn choose(&self, f: &[f32; FEATURE_DIM]) -> (DispatchClass, f32) {
        let p = self.probs(f);
        let mut best = 0usize;
        for k in 1..3 {
            if p[k] > p[best] {
                best = k;
            }
        }
        (DispatchClass::ALL[best], p[best])
    }

    /// Gradient-bandit update from the shaped reward observed AFTER
    /// dispatching `chosen` in the context `f`.
    ///
    /// With advantage a = r − baseline and dispatch probabilities p:
    ///   w[chosen] += lr · a · (1 − p[chosen]) · f
    ///   w[other]  −= lr · a · p[other] · f
    ///
    /// which is exactly ∂/∂w of a·log p[chosen] — better-than-baseline
    /// outcomes make the chosen class more likely in THIS context,
    /// worse-than-baseline outcomes make it less likely. The baseline EMA
    /// updates after the advantage is computed, with the old baseline.
    pub fn update(&mut self, chosen: &DispatchClass, reward: f32, f: &[f32; FEATURE_DIM]) {
        self.ensure_shape();
        let p = self.probs(f);
        let a = reward - self.baseline;
        let c = chosen.index();
        for k in 0..3 {
            let g = if k == c { 1.0 - p[k] } else { -p[k] };
            for i in 0..FEATURE_DIM {
                self.weights[k][i] += self.lr * a * g * f[i];
            }
        }
        self.baseline += self.baseline_rate * (reward - self.baseline);
        self.updates += 1;
        debug!(
            chosen = ?chosen,
            reward,
            advantage = a,
            baseline = self.baseline,
            p_skill = p[0],
            p_rule = p[1],
            p_knowledge = p[2],
            "router update"
        );
    }

    /// Current reward baseline (telemetry).
    pub fn baseline(&self) -> f32 {
        self.baseline
    }

    /// Total updates applied (telemetry / cold-start detection).
    pub fn updates(&self) -> u64 {
        self.updates
    }

    /// Repair weight shape after deserializing a snapshot written with an
    /// older (smaller) FEATURE_DIM or fewer classes: new entries start at
    /// zero, so old knowledge is preserved and new features begin neutral.
    fn ensure_shape(&mut self) {
        self.weights.resize_with(3, || vec![0.0; FEATURE_DIM]);
        for row in &mut self.weights {
            row.resize(FEATURE_DIM, 0.0);
        }
    }
}

impl Default for LinearRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn novel_inputs() -> RouterInputs {
        RouterInputs {
            n_changes_net_ambient: 3,
            n_object_events: 2,
            global_ira: 0.2,
            ira_trend: -0.4,
            predicate_progress: 0.1,
            predicate_progress_delta: 0.0,
            mechanic_known: false,
            control_map_size: 0,
            targets_remaining: 8,
            novel_state: true,
            blocked: true,
            steps_since_level_advance: 120,
        }
    }

    fn known_mechanic_inputs() -> RouterInputs {
        RouterInputs {
            n_changes_net_ambient: 1,
            n_object_events: 1,
            global_ira: 0.9,
            ira_trend: 0.1,
            predicate_progress: 0.6,
            predicate_progress_delta: 0.05,
            mechanic_known: true,
            control_map_size: 12,
            targets_remaining: 3,
            novel_state: false,
            blocked: false,
            steps_since_level_advance: 4,
        }
    }

    #[test]
    fn features_are_bounded_with_unit_bias() {
        // Even absurd upstream values must produce bounded features —
        // the squash/clamp contract is what keeps one weird step from
        // destroying the weights.
        let extreme = RouterInputs {
            n_changes_net_ambient: u32::MAX,
            n_object_events: 1_000_000,
            global_ira: 17.0,
            ira_trend: -99.0,
            predicate_progress: -5.0,
            predicate_progress_delta: 42.0,
            mechanic_known: true,
            control_map_size: u32::MAX,
            targets_remaining: u32::MAX,
            novel_state: true,
            blocked: true,
            steps_since_level_advance: u32::MAX,
        };
        for inputs in [novel_inputs(), known_mechanic_inputs(), extreme] {
            let f = DeltaFeatures::extract(&inputs);
            assert_eq!(f[0], 1.0, "feature 0 is the bias and must be exactly 1");
            for (i, v) in f.iter().enumerate() {
                assert!(
                    (-1.0..=1.0).contains(v),
                    "feature {i} = {v} escaped [-1,1]"
                );
                assert!(v.is_finite(), "feature {i} not finite");
            }
        }
    }

    #[test]
    fn squash_is_monotone_in_counts() {
        let mut prev = -1.0f32;
        for n in [0u32, 1, 2, 5, 20, 1000] {
            let inp = RouterInputs {
                n_changes_net_ambient: n,
                ..Default::default()
            };
            let v = DeltaFeatures::extract(&inp)[1];
            assert!(v > prev, "squash must be strictly increasing in the count");
            prev = v;
        }
    }

    #[test]
    fn softmax_probs_sum_to_one() {
        let r = LinearRouter::new();
        for inputs in [novel_inputs(), known_mechanic_inputs(), RouterInputs::default()] {
            let f = DeltaFeatures::extract(&inputs);
            let p = r.probs(&f);
            let sum: f32 = p.iter().sum();
            assert!((sum - 1.0).abs() < 1e-5, "probs sum to {sum}");
            for pk in p {
                assert!(pk > 0.0 && pk < 1.0, "softmax prob {pk} outside (0,1)");
            }
        }
    }

    #[test]
    fn choose_returns_argmax_with_its_probability() {
        let r = LinearRouter::new();
        let f = DeltaFeatures::extract(&novel_inputs());
        let p = r.probs(&f);
        let (class, conf) = r.choose(&f);
        let max = p.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(conf, max);
        assert_eq!(p[class.index()], conf);
    }

    #[test]
    fn single_positive_update_shifts_mass_toward_chosen() {
        let mut r = LinearRouter::new();
        let f = DeltaFeatures::extract(&novel_inputs());
        let before = r.probs(&f)[DispatchClass::Knowledge.index()];
        r.update(&DispatchClass::Knowledge, 1.0, &f);
        let after = r.probs(&f)[DispatchClass::Knowledge.index()];
        assert!(
            after > before,
            "positive advantage must raise the chosen class ({before} -> {after})"
        );
        assert_eq!(r.updates(), 1);
        assert!(r.baseline() > 0.0, "baseline EMA must track positive reward");
    }

    /// The core acceptance test: train ONE router on two interleaved
    /// contexts — novel states where Knowledge pays, known-mechanic
    /// states where Skill pays — and verify it learns CONTEXT-SENSITIVE
    /// dispatch, not just a global favorite. Updates are fed round-robin
    /// across classes (off-policy teacher) so every class sees both
    /// contexts and the advantage signal does the separating.
    #[test]
    fn learns_context_sensitive_dispatch() {
        let mut r = LinearRouter::new();
        let f_novel = DeltaFeatures::extract(&novel_inputs());
        let f_known = DeltaFeatures::extract(&known_mechanic_inputs());

        for step in 0..200 {
            let chosen = DispatchClass::ALL[step % 3];
            // Novel context: reward only Knowledge.
            let reward = if chosen == DispatchClass::Knowledge { 1.0 } else { 0.0 };
            r.update(&chosen, reward, &f_novel);
            // Known-mechanic context: reward only Skill.
            let reward = if chosen == DispatchClass::Skill { 1.0 } else { 0.0 };
            r.update(&chosen, reward, &f_known);
        }

        let (novel_class, novel_conf) = r.choose(&f_novel);
        let (known_class, known_conf) = r.choose(&f_known);
        assert_eq!(
            novel_class,
            DispatchClass::Knowledge,
            "novel context must route to Knowledge (conf {novel_conf})"
        );
        assert_eq!(
            known_class,
            DispatchClass::Skill,
            "known-mechanic context must route to Skill (conf {known_conf})"
        );
        // The router should be confident, not just barely past 1/3.
        assert!(novel_conf > 0.5, "novel confidence too weak: {novel_conf}");
        assert!(known_conf > 0.5, "known confidence too weak: {known_conf}");
    }

    #[test]
    fn negative_advantage_pushes_away_from_chosen() {
        let mut r = LinearRouter::new();
        let f = DeltaFeatures::extract(&known_mechanic_inputs());
        // Drive the baseline up first so a zero reward is a NEGATIVE advantage.
        for _ in 0..50 {
            r.update(&DispatchClass::Rule, 1.0, &f);
        }
        let before = r.probs(&f)[DispatchClass::Rule.index()];
        r.update(&DispatchClass::Rule, 0.0, &f);
        let after = r.probs(&f)[DispatchClass::Rule.index()];
        assert!(
            after < before,
            "below-baseline reward must lower the chosen class ({before} -> {after})"
        );
    }

    #[test]
    fn serde_roundtrip_preserves_learned_routing() {
        let mut r = LinearRouter::new();
        let f_novel = DeltaFeatures::extract(&novel_inputs());
        let f_known = DeltaFeatures::extract(&known_mechanic_inputs());
        for step in 0..120 {
            let chosen = DispatchClass::ALL[step % 3];
            let reward = if chosen == DispatchClass::Knowledge { 1.0 } else { 0.0 };
            r.update(&chosen, reward, &f_novel);
            let reward = if chosen == DispatchClass::Skill { 1.0 } else { 0.0 };
            r.update(&chosen, reward, &f_known);
        }

        let json = serde_json::to_string(&r).expect("serialize");
        let r2: LinearRouter = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(r2.updates(), r.updates());
        assert!((r2.baseline() - r.baseline()).abs() < 1e-6);
        // Behavior, not just fields: the restored router must dispatch
        // identically — this is what "knowledge persists across sessions"
        // actually requires.
        for f in [&f_novel, &f_known] {
            let (c1, p1) = r.choose(f);
            let (c2, p2) = r2.choose(f);
            assert_eq!(c1, c2);
            assert!((p1 - p2).abs() < 1e-6);
        }

        // DispatchClass itself roundtrips (it is embedded in decision logs).
        let dc = DispatchClass::Knowledge;
        let s = serde_json::to_string(&dc).unwrap();
        let dc2: DispatchClass = serde_json::from_str(&s).unwrap();
        assert_eq!(dc, dc2);
    }

    #[test]
    fn deserializing_smaller_feature_dim_loads_cleanly() {
        // A snapshot written before FEATURE_DIM grew must still load and
        // update without panicking — new feature weights start at zero.
        let json = format!(
            "{{\"weights\":[[0.5],[0.1],[0.2]],\"baseline\":0.3,\"lr\":0.15,\
             \"baseline_rate\":0.05,\"updates\":7}}"
        );
        let mut r: LinearRouter = serde_json::from_str(&json).expect("old snapshot loads");
        let f = DeltaFeatures::extract(&novel_inputs());
        let p = r.probs(&f);
        assert!((p.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        r.update(&DispatchClass::Skill, 1.0, &f); // must not panic
        assert_eq!(r.updates(), 8);
    }

    #[test]
    fn deterministic_init_no_rng() {
        // Two fresh routers must agree exactly — replayability depends on it.
        let a = LinearRouter::new();
        let b = LinearRouter::new();
        let f = DeltaFeatures::extract(&novel_inputs());
        assert_eq!(a.probs(&f), b.probs(&f));
        assert_eq!(a.choose(&f).0, b.choose(&f).0);
    }
}
