//! # 002 | Vortex LLM: training corpora with provenance
//!
//! Reads files, so it lives here rather than in `vortex_api::lm`; the
//! language model itself trains on the lines this hands it.
//!
//! A [`Corpus`] is lines of text plus a provenance manifest
//! (`vortex_api::provenance`) that content-addresses every line to a declared source with a
//! licence. A deterministic hash split reserves a held-out slice (never
//! trained on) for perplexity.

use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use vortex_api::provenance::{Manifest, ManifestBuilder, Source};
use serde::Serialize;
use vortex_kernel::fnv1a64;

pub struct Corpus {
    pub name: String,
    pub train: Vec<String>,
    pub heldout: Vec<String>,
    pub manifest: Manifest,
}

#[derive(Clone, Debug, Serialize)]
pub struct CorpusReport {
    pub name: String,
    pub train_lines: usize,
    pub heldout_lines: usize,
    pub bytes_read: usize,
    pub provenance_verified: bool,
    pub undeclared_sources: Vec<String>,
    pub merkle_root: String,
}

impl Corpus {
    /// Read up to `max_bytes` of a line-per-sentence file. Every
    /// `heldout_every`-th line (by content hash) is held out.
    pub fn from_lines_file(
        path: &Path,
        max_bytes: usize,
        heldout_every: u64,
        source: Source,
        dataset: &str,
    ) -> std::io::Result<Self> {
        let f = std::fs::File::open(path)?;
        let reader = BufReader::new(f.take(max_bytes as u64));
        let mut lines = Vec::new();
        for line in reader.lines() {
            // The byte cap can cut the final line mid-character.
            let Ok(line) = line else { break };
            let t = line.trim();
            if t.len() >= 3 {
                lines.push(t.to_string());
            }
        }
        Ok(Self::from_lines(lines, heldout_every, source, dataset))
    }

    pub fn from_lines(lines: Vec<String>, heldout_every: u64, source: Source, dataset: &str) -> Self {
        let source_id = source.id.clone();
        let mut b = ManifestBuilder::new(dataset).source(source);
        let (mut train, mut heldout) = (Vec::new(), Vec::new());
        for l in lines {
            b.record(&l, &source_id);
            if heldout_every > 0 && fnv1a64(l.as_bytes()).is_multiple_of(heldout_every) {
                heldout.push(l);
            } else {
                train.push(l);
            }
        }
        Self { name: dataset.to_string(), train, heldout, manifest: b.build() }
    }

    pub fn report(&self) -> CorpusReport {
        CorpusReport {
            name: self.name.clone(),
            train_lines: self.train.len(),
            heldout_lines: self.heldout.len(),
            bytes_read: self.train.iter().chain(&self.heldout).map(|l| l.len() + 1).sum(),
            provenance_verified: self.manifest.verify().is_ok() && self.manifest.undeclared_sources().is_empty(),
            undeclared_sources: self.manifest.undeclared_sources(),
            merkle_root: self.manifest.merkle_root.clone(),
        }
    }
}

/// The AI2 ARC Corpus: 14M science sentences released with the ARC dataset
/// (Clark et al. 2018), CC BY-SA 4.0.
pub fn arc_corpus_source() -> Source {
    Source::new("ai2-arc-corpus", "Allen Institute for AI", "CC-BY-SA-4.0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_verifies_and_split_is_deterministic() {
        let lines: Vec<String> = (0..500).map(|i| format!("sentence number {i} about rocks")).collect();
        let a = Corpus::from_lines(lines.clone(), 10, arc_corpus_source(), "t");
        let b = Corpus::from_lines(lines, 10, arc_corpus_source(), "t");
        assert_eq!(a.heldout, b.heldout);
        assert!(!a.heldout.is_empty() && a.heldout.len() < 120);
        let r = a.report();
        assert!(r.provenance_verified, "{r:?}");
        assert_eq!(r.train_lines + r.heldout_lines, 500);
    }
}
