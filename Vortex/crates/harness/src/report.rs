//! Results, the Vortex Index, terminal tables and the append-only ledger.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use vortex_kernel::Mechanism;

use crate::Capability;

#[derive(Clone, Debug, Serialize)]
pub struct SuiteResult {
    pub id: String,
    pub capability: Capability,
    pub source: String,
    pub sealed: bool,
    pub n: usize,
    pub accuracy: f64,
    pub ci95: (f64, f64),
    pub chance: f64,
    /// (accuracy - chance) / (1 - chance). 0 = chance, 1 = perfect.
    pub normalized: f64,
    /// Wilson lower bound above chance.
    pub above_chance: bool,
    pub abstain_rate: f64,
    /// Mean squared error of stated confidence vs score. Lower is better;
    /// always answering with confidence 0.5 scores 0.25.
    pub brier: f64,
    pub latency_ms_mean: f64,
    pub latency_ms_p95: f64,
    pub wall_seconds: f64,
    pub mechanism: Mechanism,
    pub mechanisms: BTreeMap<Mechanism, usize>,
    pub extra: BTreeMap<String, f64>,
    /// Up to 50 failing ids (omitted for sealed suites).
    pub failed_ids: Vec<String>,
    /// Fraction of items with n-gram overlap against the training corpus.
    pub contamination_rate: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub run_id: String,
    pub unix_time: u64,
    pub git_commit: String,
    pub constitution: String,
    pub agent: String,
    pub seed: u64,
    pub config: serde_json::Value,
    pub suites: Vec<SuiteResult>,
    /// Mean normalised score over every registered suite, with unattempted
    /// suites contributing zero.
    pub vortex_index: f64,
    /// Fraction of suites with accuracy significantly above chance.
    pub coverage: f64,
    pub by_capability: BTreeMap<Capability, f64>,
    pub notes: Vec<String>,
}

impl Report {
    pub fn new(agent: String, seed: u64, constitution: String, config: serde_json::Value, suites: Vec<SuiteResult>, notes: Vec<String>) -> Self {
        let unix_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let n = suites.len().max(1) as f64;
        let vortex_index = suites.iter().map(|s| s.normalized).sum::<f64>() / n;
        let coverage = suites.iter().filter(|s| s.above_chance).count() as f64 / n;
        let mut caps: BTreeMap<Capability, (f64, usize)> = BTreeMap::new();
        for s in &suites {
            let e = caps.entry(s.capability).or_insert((0.0, 0));
            e.0 += s.normalized;
            e.1 += 1;
        }
        let by_capability = caps.into_iter().map(|(k, (sum, c))| (k, sum / c as f64)).collect();
        Self {
            run_id: format!("{unix_time:x}-{seed:x}"),
            unix_time,
            git_commit: git_commit(),
            constitution,
            agent,
            seed,
            config,
            suites,
            vortex_index,
            coverage,
            by_capability,
            notes,
        }
    }

    pub fn table(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "{:<22} {:<13} {:>5} {:>7} {:>15} {:>6} {:>7} {:>6} {:>6} {:>8} {:<10}\n",
            "suite", "capability", "n", "score", "95% CI", "chance", "norm", "abst", "brier", "ms/item", "mechanism"
        ));
        for r in &self.suites {
            s.push_str(&format!(
                "{:<22} {:<13} {:>5} {:>6.1}% {:>6.1}-{:<6.1}% {:>5.1}% {:>+7.3} {:>5.0}% {:>6.3} {:>8.2} {:<10}{}\n",
                r.id,
                format!("{:?}", r.capability),
                r.n,
                r.accuracy * 100.0,
                r.ci95.0 * 100.0,
                r.ci95.1 * 100.0,
                r.chance * 100.0,
                r.normalized,
                r.abstain_rate * 100.0,
                r.brier,
                r.latency_ms_mean,
                format!("{:?}", r.mechanism).to_lowercase(),
                if r.sealed { "  [sealed]" } else { "" }
            ));
        }
        s.push_str(&format!(
            "\nVortex Index (mean normalised over {} suites, unattempted = 0): {:+.3}\nCoverage (suites significantly above chance): {:.0}%\n",
            self.suites.len(),
            self.vortex_index,
            self.coverage * 100.0
        ));
        s
    }

    /// Append to the JSONL ledger. Never rewrites earlier lines.
    pub fn append_to_ledger(&self, path: &Path) -> std::io::Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
        let line = serde_json::to_string(self).map_err(std::io::Error::other)?;
        writeln!(f, "{line}")
    }
}

fn git_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}
