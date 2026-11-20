//! # 002 | Vortex LLM
//!
//! Language modelling for Vortex, behind the [`LanguageModel`] trait so the
//! backend can be replaced without touching callers.
//!
//! **v1 backend: [`NgramLm`]**, an interpolated Kneser-Ney n-gram model
//! (orders 2-4) with exact packed keys (no hash collisions). It is
//! the honest floor: a well-understood statistical LM, trained here from
//! scratch in pure Rust, that every later neural backend (the Burn
//! transformer and diffusion LM carried over from `aimodel`) must beat on the
//! same held-out perplexity and the same harness suites before it replaces it.
//!
//! Training data enters only through `vortex_lab::corpus::Corpus` (it reads
//! files, so it lives in `vortex-lab`), which builds a
//! [`crate::provenance::Manifest`] over every record. The constitution (A7) refuses
//! training when that manifest does not verify. [`contamination`] measures
//! n-gram overlap between the training text and every evaluation set, and the
//! harness reports it. Training itself ([`NgramLm::train`]) takes lines from
//! the caller and does no I/O.

pub mod contamination;
pub mod ngram;
pub mod tokenize;

pub use ngram::{NgramConfig, NgramLm};

/// How multiple-choice continuations are compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceScoring {
    /// Total log-probability of the continuation (favours short answers).
    Sum,
    /// Mean log-probability per token.
    MeanPerToken,
    /// Pointwise mutual information: log P(choice | context) minus
    /// log P(choice | no context). Removes the prior of the answer text.
    Pmi,
}

impl ChoiceScoring {
    pub const ALL: [ChoiceScoring; 3] = [ChoiceScoring::Sum, ChoiceScoring::MeanPerToken, ChoiceScoring::Pmi];
}

pub trait LanguageModel: Send + Sync {
    fn name(&self) -> String;

    /// Natural-log probability of `continuation` given `context`, summed over
    /// tokens, plus the number of tokens scored.
    fn logprob(&self, context: &str, continuation: &str) -> (f64, usize);

    /// Score each choice; returns (best index, softmax probabilities).
    fn choose(&self, context: &str, choices: &[String], scoring: ChoiceScoring) -> (usize, Vec<f64>) {
        let scores: Vec<f64> = choices
            .iter()
            .map(|c| {
                let (lp, n) = self.logprob(context, c);
                match scoring {
                    ChoiceScoring::Sum => lp,
                    ChoiceScoring::MeanPerToken => lp / n.max(1) as f64,
                    ChoiceScoring::Pmi => lp - self.logprob("", c).0,
                }
            })
            .collect();
        let best = scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        (best, softmax(&scores))
    }
}

pub fn softmax(xs: &[f64]) -> Vec<f64> {
    if xs.is_empty() {
        return vec![];
    }
    let m = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let e: Vec<f64> = xs.iter().map(|x| (x - m).exp()).collect();
    let s: f64 = e.iter().sum();
    e.into_iter().map(|x| x / s).collect()
}
