//! # 001 | Vortex Orchestrator: the lifecycle
//!
//! The seven disciplines, end to end: Acquisition, Curation, Provenance,
//! Training, Reinforcement, Evaluation, Publishing. Every stage is
//! recorded; a stage the constitution refuses stops the run. A sealed set
//! that fails its hash pin stops the run before anything is trained or
//! scored.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use serde_json::json;
use vortex_api::Vortex;
use vortex_api::constitution::{Constitution, Proposal};
use vortex_api::lm::{ChoiceScoring, LanguageModel, NgramConfig, NgramLm, contamination};
use vortex_api::pipeline::{Discipline, Lifecycle};
use vortex_api::rsi::Trial;
use vortex_harness::suites::{self, SuiteConfig, choice};
use vortex_harness::{Report, Suite, SuiteResult, run_suite};
use vortex_kernel::Answer;

use crate::corpus::{Corpus, arc_corpus_source};
use crate::economics::{Assumptions, CostReport, Workload};

#[derive(Clone, Debug, Serialize)]
pub struct EvalOptions {
    pub suites: SuiteConfig,
    /// The training text (one sentence per line).
    pub corpus_path: PathBuf,
    pub lm_bytes: usize,
    pub lm: NgramConfig,
    pub no_lm: bool,
    pub threads: usize,
    /// Only run suites whose id is listed (empty = all).
    pub only: Vec<String>,
    pub eval_dir: PathBuf,
    pub check_contamination: bool,
}

pub struct EvalOutcome {
    pub lifecycle: Lifecycle,
    pub report: Report,
    pub cost: CostReport,
    pub warnings: Vec<String>,
}

fn dev_suites(o: &EvalOptions) -> Vec<Box<dyn Suite>> {
    let d = &o.suites.data_dir;
    let mut v: Vec<Box<dyn Suite>> = Vec::new();
    for (id, f) in [("arc-easy-dev", "arc/ARC-V1-Feb2018-2/ARC-Easy/ARC-Easy-Dev.jsonl"), ("arc-challenge-dev", "arc/ARC-V1-Feb2018-2/ARC-Challenge/ARC-Challenge-Dev.jsonl")] {
        if let Ok(s) = choice::ai2_arc_all(id, &d.join(f)) {
            v.push(Box::new(s));
        }
    }
    v
}

fn outcomes(agent: &Vortex, suites: &[Box<dyn Suite>], threads: usize) -> Vec<bool> {
    suites
        .iter()
        .flat_map(|s| {
            let (_, rs) = run_suite(agent, s.as_ref(), threads);
            rs.iter().enumerate().map(|(i, r)| s.grade(i, r) >= 0.999).collect::<Vec<_>>()
        })
        .collect()
}

pub fn run(o: &EvalOptions, mut log: impl FnMut(&str)) -> Result<EvalOutcome, String> {
    let constitution = Constitution::v1();
    let mut life = Lifecycle::new(&constitution);
    let mut warnings = Vec::new();
    // Suites load first: a sealed set failing its MANIFEST.sha256 pin aborts
    // the run before anything is trained, tuned, scored or ledgered.
    let (mut all, w) = suites::standard(&o.suites).map_err(|e| format!("sealed-set integrity check failed, nothing was scored: {e}"))?;
    life.begin();

    // ---- Acquisition / Curation / Provenance / Training --------------------
    let mut lm: Option<Arc<NgramLm>> = None;
    let mut corpus_train: Vec<String> = Vec::new();
    if o.no_lm {
        for d in [Discipline::Acquisition, Discipline::Curation, Discipline::Provenance, Discipline::Training] {
            life.skipped(d, "--no-lm");
        }
    } else {
        log(&format!("acquisition: reading {} MB of {}", o.lm_bytes / (1 << 20), o.corpus_path.display()));
        let corpus = Corpus::from_lines_file(&o.corpus_path, o.lm_bytes, 100, arc_corpus_source(), "ai2-arc-corpus-slice").map_err(|e| format!("{}: {e}", o.corpus_path.display()))?;
        let rep = corpus.report();
        life.done(Discipline::Acquisition, json!({"source": o.corpus_path.display().to_string(), "bytes": rep.bytes_read, "license": "CC-BY-SA-4.0"}));
        life.done(Discipline::Curation, json!({"train_lines": rep.train_lines, "heldout_lines": rep.heldout_lines, "split": "fnv1a(line) % 100 == 0 held out"}));
        let ruling = constitution.review(&Proposal::Train { provenance_verified: rep.provenance_verified, undeclared_sources: rep.undeclared_sources.len(), uses_eval_split: false });
        if !life.gated(Discipline::Provenance, ruling, json!({"verified": rep.provenance_verified, "merkle_root": rep.merkle_root})) {
            return Err(format!("constitution refused training:\n{}", life.summary()));
        }
        log("training: interpolated Kneser-Ney n-gram");
        let t0 = Instant::now();
        let model = NgramLm::train(corpus.train.iter().map(String::as_str), &o.lm);
        let ppl = model.perplexity(corpus.heldout.iter().map(String::as_str));
        life.done(Discipline::Training, json!({"model": LanguageModel::name(&model), "order": o.lm.order, "heldout_perplexity": ppl, "seconds": t0.elapsed().as_secs_f64(), "ngrams_per_order": model.stats.ngrams_per_order}));
        corpus_train = corpus.train;
        lm = Some(Arc::new(model));
    }

    // ---- Reinforcement: choose LM scoring on the dev split only -------------
    let mut agent = match &lm {
        Some(lm) => Vortex::new().with_lm(lm.clone()).with_scoring(ChoiceScoring::Sum),
        None => Vortex::new().with_scoring(ChoiceScoring::Sum),
    };
    if lm.is_some() {
        let dev = dev_suites(o);
        if dev.is_empty() {
            life.skipped(Discipline::Reinforcement, "dev split not found");
        } else {
            log("reinforcement: improvement gate over LM scoring modes on ARC dev");
            let trial = |s: ChoiceScoring| Trial { name: format!("{s:?}"), config: json!({"scoring": s}), outcomes: outcomes(&agent.clone().with_scoring(s), &dev, o.threads) };
            let incumbent = trial(ChoiceScoring::Sum);
            let candidates = vec![trial(ChoiceScoring::MeanPerToken), trial(ChoiceScoring::Pmi)];
            // Scoring mode only affects LM multiple choice; the safety lanes
            // do not use the LM, so no candidate can regress them.
            let sel = vortex_api::rsi::select(&constitution, incumbent, candidates, |_| 0);
            let _ = crate::ledger::record(&o.eval_dir.join("rsi_ledger.jsonl"), &sel);
            let chosen: ChoiceScoring = serde_json::from_value(sel.chosen.config["scoring"].clone()).unwrap_or(ChoiceScoring::Sum);
            agent = agent.with_scoring(chosen);
            life.done(
                Discipline::Reinforcement,
                json!({
                    "dev_items": sel.dev_items,
                    "incumbent": format!("{} {:.3}", sel.incumbent.name, sel.incumbent.accuracy()),
                    "candidates": sel.comparisons.iter().map(|c| format!("{} {:.3} p={:.4} {:?}", c.candidate, c.candidate_accuracy, c.evidence.p_value, c.decision)).collect::<Vec<_>>(),
                    "adopted": sel.chosen.name,
                }),
            );
        }
    } else {
        life.skipped(Discipline::Reinforcement, "no LM");
    }

    // ---- Evaluation ----------------------------------------------------------
    warnings.extend(w);
    if !o.only.is_empty() {
        all.retain(|s| o.only.iter().any(|x| x == s.id()));
    }
    // Contamination for every suite in one pass over the training text.
    let texts: Vec<(String, Vec<String>)> = if o.check_contamination && !corpus_train.is_empty() {
        all.iter().map(|s| (s.id().to_string(), s.contamination_texts())).filter(|(_, t)| !t.is_empty()).collect()
    } else {
        vec![]
    };
    let contaminated = if texts.is_empty() {
        vec![]
    } else {
        log(&format!("contamination: {} suites against the training text, one pass", texts.len()));
        let wanted: Vec<(&str, &[String])> = texts.iter().map(|(id, t)| (id.as_str(), t.as_slice())).collect();
        contamination::check_many(&wanted, corpus_train.iter().map(String::as_str))
    };
    let mut results: Vec<SuiteResult> = Vec::new();
    for s in &all {
        log(&format!("evaluating {} ({} items)", s.id(), s.items().len()));
        let (mut r, responses) = run_suite(&agent, s.as_ref(), o.threads);
        if let Some(c) = contaminated.iter().find(|c| c.suite == s.id()) {
            r.contamination_rate = Some(c.rate);
            let clean: Vec<usize> = (0..responses.len()).filter(|i| !c.flags.get(*i).copied().unwrap_or(false)).collect();
            if !clean.is_empty() {
                let acc = clean.iter().map(|&i| s.grade(i, &responses[i])).sum::<f64>() / clean.len() as f64;
                r.extra.insert("clean_accuracy".into(), acc);
                r.extra.insert("clean_items".into(), clean.len() as f64);
            }
        }
        // A suite answered entirely by abstention is "not attempted".
        if responses.iter().all(|x| matches!(x.answer, Answer::Abstain { .. })) {
            r.extra.insert("not_attempted".into(), 1.0);
        }
        results.push(r);
    }
    life.done(Discipline::Evaluation, json!({"suites": results.len(), "warnings": warnings}));

    // ---- Publishing (local ledger only) ---------------------------------------
    let report = Report::new(
        vortex_harness::Agent::name(&agent),
        o.suites.seed,
        constitution.hash().to_string(),
        serde_json::to_value(o).unwrap_or_default(),
        results,
        vec![
            "Learned = n-gram LM; Search = DSL program search; Engineered = hand-written solver or rules.".into(),
            "Unattempted suites score 0 in the Vortex Index by design.".into(),
        ],
    );
    report.append_to_ledger(&o.eval_dir.join("ledger.jsonl")).map_err(|e| e.to_string())?;
    let _ = std::fs::write(o.eval_dir.join("latest_report.json"), serde_json::to_string_pretty(&report).unwrap_or_default());
    life.done(Discipline::Publishing, json!({"ledger": o.eval_dir.join("ledger.jsonl").display().to_string(), "outward": "not published; requires human approval (A1)"}));

    // ---- Cost per request from measured latency --------------------------------
    let workloads: Vec<Workload> = report
        .suites
        .iter()
        .filter(|s| s.abstain_rate < 1.0)
        .map(|s| Workload { class: s.id.clone(), cpu_ms: s.latency_ms_mean, measured_by: format!("harness run {}, {} items", report.run_id, s.n) })
        .collect();
    let cost = crate::economics::price(&Assumptions::default(), &workloads);
    let _ = std::fs::write(o.eval_dir.join("cost_report.json"), serde_json::to_string_pretty(&cost).unwrap_or_default());

    Ok(EvalOutcome { lifecycle: life, report, cost, warnings })
}
