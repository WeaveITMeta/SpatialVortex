//! Provenance manifest: what constitution article A7 asks for before any
//! training run.
//!
//! Every training record is content-addressed (SHA-256) and tied to a
//! *declared* source with a holder and a licence. The records are sealed
//! under one Merkle root, so a manifest that has been altered after it was
//! built no longer verifies, and a record attributed to a source nobody
//! declared is reported by name. The constitution refuses training unless
//! [`Manifest::verify`] passes and [`Manifest::undeclared_sources`] is empty.
//!
//! Pure computation, like the rest of this crate: it hashes what it is
//! given. Reading the corpus is the caller's job.

use serde::Serialize;
use sha2::{Digest, Sha256};

type Hash = [u8; 32];

/// Where records came from, and under what terms.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Source {
    pub id: String,
    pub holder: String,
    pub license: String,
}

impl Source {
    pub fn new(id: impl Into<String>, holder: impl Into<String>, license: impl Into<String>) -> Self {
        Self { id: id.into(), holder: holder.into(), license: license.into() }
    }
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub dataset: String,
    pub sources: Vec<Source>,
    /// Hex SHA-256 Merkle root over every record, in the order recorded.
    pub merkle_root: String,
    /// Source ids as the records named them; a record's `u32` indexes this.
    names: Vec<String>,
    /// (content hash, source) per record.
    records: Vec<(Hash, u32)>,
}

pub struct ManifestBuilder {
    dataset: String,
    sources: Vec<Source>,
    names: Vec<String>,
    records: Vec<(Hash, u32)>,
}

fn hex(h: &Hash) -> String {
    h.iter().map(|b| format!("{b:02x}")).collect()
}

/// Leaf and node hashes are domain-separated (0x00 / 0x01), so a leaf can
/// never be passed off as an inner node. An odd node is carried up unchanged.
fn merkle_root(names: &[String], records: &[(Hash, u32)]) -> Hash {
    let mut level: Vec<Hash> = records
        .iter()
        .map(|(content, source)| {
            let mut h = Sha256::new();
            h.update([0u8]);
            h.update(content);
            h.update(names.get(*source as usize).map(String::as_bytes).unwrap_or_default());
            h.finalize().into()
        })
        .collect();
    if level.is_empty() {
        return Sha256::digest(b"").into();
    }
    while level.len() > 1 {
        level = level
            .chunks(2)
            .map(|pair| match pair {
                [l, r] => {
                    let mut h = Sha256::new();
                    h.update([1u8]);
                    h.update(l);
                    h.update(r);
                    h.finalize().into()
                }
                _ => pair[0],
            })
            .collect();
    }
    level[0]
}

impl ManifestBuilder {
    pub fn new(dataset: impl Into<String>) -> Self {
        Self { dataset: dataset.into(), sources: Vec::new(), names: Vec::new(), records: Vec::new() }
    }

    /// Declare a source. Records may only be attributed to declared sources.
    pub fn source(mut self, source: Source) -> Self {
        self.sources.push(source);
        self
    }

    /// Record one piece of content as coming from `source_id`. Recording
    /// under an undeclared id is allowed here and reported by the manifest:
    /// the builder never hides where something claimed to come from.
    pub fn record(&mut self, content: &str, source_id: &str) {
        let index = match self.names.iter().position(|n| n == source_id) {
            Some(i) => i,
            None => {
                self.names.push(source_id.to_string());
                self.names.len() - 1
            }
        };
        self.records.push((Sha256::digest(content.as_bytes()).into(), index as u32));
    }

    pub fn build(self) -> Manifest {
        let merkle_root = hex(&merkle_root(&self.names, &self.records));
        Manifest { dataset: self.dataset, sources: self.sources, merkle_root, names: self.names, records: self.records }
    }
}

impl Manifest {
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// The records still hash to the sealed root, and each names a source.
    pub fn verify(&self) -> Result<(), String> {
        if let Some((i, _)) = self.records.iter().enumerate().find(|(_, (_, s))| *s as usize >= self.names.len()) {
            return Err(format!("record {i} names no source"));
        }
        let actual = hex(&merkle_root(&self.names, &self.records));
        if actual != self.merkle_root {
            return Err(format!("merkle root mismatch: sealed {}, records hash to {actual}", self.merkle_root));
        }
        Ok(())
    }

    /// Source ids that records were attributed to but nobody declared.
    pub fn undeclared_sources(&self) -> Vec<String> {
        let mut out: Vec<String> = self.names.iter().filter(|n| !self.sources.iter().any(|s| &s.id == *n)).cloned().collect();
        out.sort();
        out
    }

    /// Whether this exact content was recorded.
    pub fn contains(&self, content: &str) -> bool {
        let h: Hash = Sha256::digest(content.as_bytes()).into();
        self.records.iter().any(|(c, _)| *c == h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(lines: &[&str], source_id: &str) -> Manifest {
        let mut b = ManifestBuilder::new("t").source(Source::new("declared", "Someone", "CC-BY-4.0"));
        for l in lines {
            b.record(l, source_id);
        }
        b.build()
    }

    #[test]
    fn verifies_and_detects_tampering() {
        let m = build(&["a", "b", "c", "d", "e"], "declared");
        assert_eq!((m.len(), m.merkle_root.len()), (5, 64));
        assert!(m.verify().is_ok());
        assert!(m.undeclared_sources().is_empty());
        assert!(m.contains("c") && !m.contains("z"));

        let mut swapped = m.clone();
        swapped.records[2].0[0] ^= 1;
        assert!(swapped.verify().unwrap_err().contains("merkle root mismatch"));
        let mut dropped = m.clone();
        dropped.records.pop();
        assert!(dropped.verify().is_err());
        let mut reattributed = m.clone();
        reattributed.names[0] = "elsewhere".into();
        assert!(reattributed.verify().is_err(), "the source is part of what is sealed");
        let mut dangling = m;
        dangling.records[0].1 = 9;
        assert!(dangling.verify().unwrap_err().contains("names no source"));
    }

    #[test]
    fn undeclared_sources_are_named() {
        let mut b = ManifestBuilder::new("t").source(Source::new("declared", "Someone", "CC-BY-4.0"));
        b.record("x", "declared");
        b.record("y", "scraped");
        b.record("z", "found-it");
        let m = b.build();
        assert!(m.verify().is_ok(), "the manifest is intact; it is the attribution that fails");
        assert_eq!(m.undeclared_sources(), vec!["found-it".to_string(), "scraped".to_string()]);
    }

    #[test]
    fn root_is_deterministic_and_sensitive_to_order_content_and_size() {
        let root = |lines: &[&str]| build(lines, "declared").merkle_root;
        assert_eq!(root(&["a", "b", "c"]), root(&["a", "b", "c"]));
        assert_ne!(root(&["a", "b", "c"]), root(&["a", "c", "b"]));
        assert_ne!(root(&["a", "b", "c"]), root(&["a", "b", "d"]));
        assert_ne!(root(&["a", "b"]), root(&["a", "b", "b"]));
        assert_ne!(root(&["a"]), root(&[]));
        assert!(build(&[], "declared").verify().is_ok());
    }
}
