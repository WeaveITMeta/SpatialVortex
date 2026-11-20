//! The registered suites. [`standard`] builds all of them; a suite whose
//! data is missing is reported as a warning, never silently dropped.

pub mod arcagi;
pub mod astra;
pub mod choice;
pub mod code;
pub mod gsm8k;
pub mod kinematics;
pub mod sealed;

use std::path::PathBuf;

use serde::Serialize;
use vortex_kernel::Rng;

use crate::{Item, Suite};

#[derive(Clone, Debug, Serialize)]
pub struct SuiteConfig {
    /// `benchmarks/data` (HellaSwag, ARC, MMLU, GSM8K, HumanEval, ARC-AGI).
    pub data_dir: PathBuf,
    /// `Vortex/eval/heldout`.
    pub heldout_dir: PathBuf,
    pub seed: u64,
    /// Items per sampled dataset suite.
    pub limit: usize,
    /// Instances per procedural suite.
    pub procedural: usize,
    /// ARC-AGI tasks (search is slow).
    pub arc_agi_limit: usize,
    /// HumanEval problems (each graded by running Python).
    pub code_limit: usize,
}

/// Deterministic seeded subsample, preserving nothing about file order.
pub(crate) fn sample<T>(mut v: Vec<T>, limit: usize, seed: u64, salt: &str) -> Vec<T> {
    if v.len() <= limit {
        return v;
    }
    let mut rng = Rng::new(seed ^ vortex_kernel::fnv1a64(salt.as_bytes()));
    let idx = rng.sample_indices(v.len(), limit);
    let mut keep = vec![false; v.len()];
    for i in idx {
        keep[i] = true;
    }
    let mut i = 0;
    v.retain(|_| {
        let k = keep[i];
        i += 1;
        k
    });
    v
}

pub(crate) fn items_from(ids_tasks: Vec<(String, vortex_kernel::Task)>) -> Vec<Item> {
    ids_tasks.into_iter().map(|(id, task)| Item { id, task }).collect()
}

/// Every registered suite. Order is the report order.
pub fn standard(cfg: &SuiteConfig) -> (Vec<Box<dyn Suite>>, Vec<String>) {
    let mut suites: Vec<Box<dyn Suite>> = Vec::new();
    let mut warnings = Vec::new();
    let d = &cfg.data_dir;
    let mut add = |r: Result<Box<dyn Suite>, String>| match r {
        Ok(s) => suites.push(s),
        Err(e) => warnings.push(e),
    };

    add(choice::hellaswag(&d.join("hellaswag/validation.jsonl"), cfg.limit, cfg.seed).map(boxed));
    add(choice::ai2_arc("arc-easy", &d.join("arc/ARC-V1-Feb2018-2/ARC-Easy/ARC-Easy-Test.jsonl"), cfg.limit, cfg.seed).map(boxed));
    add(choice::ai2_arc("arc-challenge", &d.join("arc/ARC-V1-Feb2018-2/ARC-Challenge/ARC-Challenge-Test.jsonl"), cfg.limit, cfg.seed).map(boxed));
    add(choice::mmlu(&d.join("mmlu/data/test"), cfg.limit, cfg.seed).map(boxed));
    add(gsm8k::load(&d.join("gsm8k/test.jsonl"), cfg.limit, cfg.seed).map(boxed));
    add(code::humaneval(&d.join("humaneval/HumanEval.jsonl"), cfg.code_limit, cfg.seed).map(boxed));
    add(arcagi::load(&d.join("arc-agi-2/arc_agi_training.json"), cfg.arc_agi_limit, cfg.seed).map(boxed));
    add(Ok(boxed(astra::procedural(cfg.procedural, cfg.seed))));
    add(Ok(boxed(kinematics::procedural(cfg.procedural, cfg.seed))));
    add(sealed::guardian(&cfg.heldout_dir.join("guardian_v1.jsonl")).map(boxed));
    add(sealed::action_risk(&cfg.heldout_dir.join("action_risk_v1.jsonl")).map(boxed));
    add(sealed::tool_use(&cfg.heldout_dir.join("tool_use_v1.jsonl")).map(boxed));
    (suites, warnings)
}

fn boxed<S: Suite + 'static>(s: S) -> Box<dyn Suite> {
    Box::new(s)
}

/// Read a file to string with a useful error.
pub(crate) fn read(path: &std::path::Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}
