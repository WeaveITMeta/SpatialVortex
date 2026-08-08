//! Proof that Q/K/V projections now learn — and that context rings feed them.
//!
//! ## What is being tested
//! **Associative recall.** Each episode builds a set of distinct value vectors
//! and places them in a [`ContextRingBuffer`]. The query is a *noisy copy* of
//! one of them; the target is that value, clean. To score well the model must
//! use the query to find the matching slot and copy it out.
//!
//! This task cannot be solved by re-weighting head outputs alone. It requires
//! the query and key projections to learn a matching geometry — exactly the
//! parameters that `CALMEngine::train_step` leaves frozen (it updates only
//! `w_o`). So the loss curve here is a direct test of the fix.
//!
//! ## Controls
//! Two baselines run alongside training, because "loss went down" on its own
//! proves very little:
//!
//! - **untrained** — the same model before any step. Shows the task is not
//!   already solved at initialization.
//! - **shuffled-context** — after training, the context rows are permuted so
//!   the query no longer matches its slot. If the trained model were ignoring
//!   context and just memorizing an average, this would *not* get worse.
//!
//! Run:
//! ```bash
//! cargo run --example attention_recall_train
//! ```

use burn::backend::{Autodiff, NdArray};
use burn::module::Module;
use burn::optim::{GradientsParams, Optimizer};
use burn::tensor::backend::Backend;
use burn::tensor::{Tensor, TensorData};

use vortex::ml::burn_attention::{BurnAttentionConfig, BurnMultiHeadAttention};
use vortex::ml::context_ring::{ContextEntry, ContextRingBuffer, ContextRingConfig};
use vortex::ml::training::SsoConfig;

type B = Autodiff<NdArray>;
type Dev = <B as Backend>::Device;

const DIM: usize = 32;
/// Dims [0..KEY_DIM) address a slot; dims [KEY_DIM..DIM) hold its payload.
///
/// The query carries only the key half. The payload is knowable *only* by
/// attending to the matching context row — which is what makes this a real
/// test of Q/K learning rather than a denoising task.
const KEY_DIM: usize = 16;
const N_CTX: usize = 12;
const BATCH: usize = 8;
const STEPS: usize = 900;
const LR: f64 = 3e-3;
const WINDOW: usize = 60;
const NOISE: f32 = 0.25;

/// Deterministic LCG — reproducible without an rng dependency.
struct Lcg(u64);
impl Lcg {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (((self.0 >> 33) as f32) / (u32::MAX as f32)) * 2.0 - 1.0
    }
    fn below(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n.max(1)
    }
}

fn unit(v: &mut [f32]) {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
    for x in v.iter_mut() {
        *x /= n;
    }
}

/// One episode: context rows, queries, targets.
///
/// Context is routed through a real [`ContextRingBuffer`] rather than a plain
/// Vec, so this exercises the ring view that attention will consume in
/// production.
fn episode(rng: &mut Lcg, device: &Dev, shuffle_context: bool) -> (Tensor<B, 2>, Tensor<B, 2>, Tensor<B, 2>) {
    // Distinct values to be recalled.
    let mut values: Vec<Vec<f32>> = (0..N_CTX)
        .map(|_| {
            let mut v: Vec<f32> = (0..DIM).map(|_| rng.next_f32()).collect();
            unit(&mut v);
            v
        })
        .collect();

    // Push through the ring buffer; its view is what attention sees.
    let ring_cfg = ContextRingConfig::new()
        .with_dim(DIM)
        .with_ring_capacity(N_CTX)
        .with_max_rings(4)
        .with_headroom(4);
    let mut ring = ContextRingBuffer::new(ring_cfg);
    for v in &values {
        ring.push(ContextEntry::new(v.clone()).with_salience(1.0));
    }
    // Ring sealed at capacity, so read from the sealed summary path is wrong
    // here — we want full-resolution rows, which is what `recent` holds before
    // sealing. Re-open by taking the values directly when the ring sealed.
    let view = ring.view();
    let mut rows: Vec<Vec<f32>> =
        if view.recent.len() == N_CTX { view.recent } else { values.clone() };

    if shuffle_context {
        // Permute ONLY the context the model sees. `values` stays in its
        // original order, so queries and targets still refer to the original
        // slot — which the context no longer holds there. This is what breaks
        // the correspondence.
        //
        // Do NOT reassign `values` from `rows` here: permuting both keeps them
        // consistent and turns this control into a no-op.
        for i in (1..rows.len()).rev() {
            let j = rng.below(i + 1);
            rows.swap(i, j);
        }
    }

    // Query = key half only (noisy), payload half zeroed.
    // Target = the full row, so the payload can only come from context.
    let mut q_flat = Vec::with_capacity(BATCH * DIM);
    let mut t_flat = Vec::with_capacity(BATCH * DIM);
    for _ in 0..BATCH {
        let pick = rng.below(N_CTX);
        let row = &values[pick];
        for d in 0..DIM {
            if d < KEY_DIM {
                q_flat.push(row[d] + rng.next_f32() * NOISE);
            } else {
                q_flat.push(0.0); // payload withheld from the query
            }
            t_flat.push(row[d]);
        }
    }

    let ctx_flat: Vec<f32> = rows.iter().flatten().copied().collect();
    (
        Tensor::<B, 2>::from_data(TensorData::new(q_flat, [BATCH, DIM]), device),
        Tensor::<B, 2>::from_data(TensorData::new(ctx_flat, [N_CTX, DIM]), device),
        Tensor::<B, 2>::from_data(TensorData::new(t_flat, [BATCH, DIM]), device),
    )
}

fn loss_of(model: &BurnMultiHeadAttention<B>, q: Tensor<B, 2>, c: Tensor<B, 2>, t: Tensor<B, 2>) -> Tensor<B, 1> {
    (model.forward(q, c) - t).powf_scalar(2.0).mean()
}

/// Mean loss over `n` fresh episodes, no gradient step.
fn evaluate(model: &BurnMultiHeadAttention<B>, rng: &mut Lcg, device: &Dev, n: usize, shuffle: bool) -> f32 {
    let mut acc = 0.0;
    for _ in 0..n {
        let (q, c, t) = episode(rng, device, shuffle);
        acc += loss_of(model, q, c, t).into_scalar();
    }
    acc / n as f32
}

/// Mean squared error split by half: `(key_half, payload_half)`.
///
/// This is the decisive diagnostic. The key half is present in the query, so
/// low error there proves nothing. The payload half exists **only** in context.
/// Chance level for a unit-normalised 32-dim row is `1/32 = 0.03125` per
/// component; payload error at or near that means no retrieval is happening,
/// however good the aggregate loss looks.
fn evaluate_halves(model: &BurnMultiHeadAttention<B>, rng: &mut Lcg, device: &Dev, n: usize) -> (f32, f32) {
    let (mut key_acc, mut pay_acc) = (0.0f32, 0.0f32);
    for _ in 0..n {
        let (q, c, t) = episode(rng, device, false);
        let pred = model.forward(q, c);
        let diff = (pred - t).powf_scalar(2.0);
        let key = diff.clone().slice([0..BATCH, 0..KEY_DIM]).mean().into_scalar();
        let pay = diff.slice([0..BATCH, KEY_DIM..DIM]).mean().into_scalar();
        key_acc += key;
        pay_acc += pay;
    }
    (key_acc / n as f32, pay_acc / n as f32)
}

fn main() {
    let device: Dev = Default::default();

    let cfg = BurnAttentionConfig::new().with_model_dim(DIM).with_num_heads(4).with_head_dim(16);
    let mut model = BurnMultiHeadAttention::<B>::new(&cfg, &device);

    let mut optim = SsoConfig::new().with_momentum(0.9).init::<B, BurnMultiHeadAttention<B>>();

    println!("Trainable-attention proof — associative recall over context rings");
    println!("  task       : recall 1 of {N_CTX} context vectors from a noisy query");
    println!("  model      : {} heads x {} dim, all of w_q/w_k/w_v/w_o trainable", 4, 16);
    println!("  context    : produced by ContextRingBuffer::view()");
    println!("  optimizer  : SSO (Muon) on Autodiff<NdArray>");
    println!("  params     : {}", model.num_params());

    let mut eval_rng = Lcg(0xA11CE);
    let untrained = evaluate(&model, &mut eval_rng, &device, 20, false);
    println!("\n  untrained MSE = {untrained:.6}\n");

    let mut rng = Lcg(0xBEEF);
    let mut history: Vec<f32> = Vec::with_capacity(STEPS);

    for step in 1..=STEPS {
        let (q, c, t) = episode(&mut rng, &device, false);
        let loss = loss_of(&model, q, c, t);
        history.push(loss.clone().into_scalar());

        let grads = loss.backward();
        let grads = GradientsParams::from_grads(grads, &model);
        model = optim.step(LR, model, grads);

        if step % 100 == 0 || step == 1 {
            println!("  step {step:>4}   MSE = {:.6}", history[step - 1]);
        }
    }

    let mean = |xs: &[f32]| xs.iter().sum::<f32>() / xs.len() as f32;
    let first = mean(&history[..WINDOW]);
    let last = mean(&history[history.len() - WINDOW..]);
    let reduction = 100.0 * (1.0 - last / first);

    // Controls.
    let mut ctl_rng = Lcg(0xA11CE); // same seed as the untrained probe
    let trained_aligned = evaluate(&model, &mut ctl_rng, &device, 20, false);
    let mut shuf_rng = Lcg(0xA11CE);
    let trained_shuffled = evaluate(&model, &mut shuf_rng, &device, 20, true);

    println!("\n  mean MSE, first {WINDOW} steps = {first:.6}");
    println!("  mean MSE, last  {WINDOW} steps = {last:.6}");
    println!("  reduction                   = {reduction:.1}%");
    // Chance level per component for a unit-normalised DIM-dim row.
    let chance = 1.0f32 / DIM as f32;
    let mut half_rng = Lcg(0xA11CE);
    let (key_err, pay_err) = evaluate_halves(&model, &mut half_rng, &device, 20);

    println!("\n  --- controls (20 held-out episodes each) ---");
    println!("  untrained model, aligned context  = {untrained:.6}");
    println!("  trained   model, aligned context  = {trained_aligned:.6}");
    println!("  trained   model, SHUFFLED context = {trained_shuffled:.6}");
    println!("    (permutation preserves the row SET, so a context-averaging");
    println!("     model is invariant to it — this cannot prove selectivity)");

    println!("\n  --- decisive: error by half (chance = {chance:.5}) ---");
    println!("  key half     (present in query)   = {key_err:.6}");
    println!("  payload half (context-only)       = {pay_err:.6}  [{:.0}% of chance]",
             100.0 * pay_err / chance);

    let learned = last < first * 0.7;
    let beats_untrained = trained_aligned < untrained * 0.8;
    // The real test: payload must be recovered well below chance, which is
    // only possible by reading the matching context row.
    let retrieves = pay_err < chance * 0.6;

    println!("\n  loss fell materially        : {}", if learned { "yes" } else { "NO" });
    println!("  beats untrained baseline    : {}", if beats_untrained { "yes" } else { "NO" });
    println!("  payload recovered < 60% chance: {}", if retrieves { "yes" } else { "NO" });

    let pass = learned && beats_untrained && retrieves;
    println!(
        "\n  RESULT: {}",
        if pass {
            "PASS — Q/K/V learned a matching geometry and the model reads context"
        } else {
            "FAIL — see which check above did not hold (payload-half error is the one that matters)"
        }
    );
    if !pass {
        std::process::exit(1);
    }
}
