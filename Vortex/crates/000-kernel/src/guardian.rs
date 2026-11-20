//! Eustress AI Guardian Policy v1.2 taxonomy
//! (`EustressEngine/docs/architecture/AI_GUARDIAN_POLICY_v1.0.md`).
//!
//! Vortex's content-safety lane classifies into exactly the engine's publish
//! gate categories, so a Vortex verdict and a Guardian Judge verdict on the
//! same content are directly comparable.

use serde::{Deserialize, Serialize};

/// The eight prohibited categories. Content that clearly falls into one of
/// these is a `Reject`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum GuardianCategory {
    /// CSAM and child exploitation. Zero tolerance.
    G1,
    /// Terrorism, violent extremism, real-world mass-violence incitement.
    G2,
    /// Actionable instructions for real-world serious crime.
    G3,
    /// Non-consensual intimate imagery and real-person deepfakes.
    G4,
    /// Doxxing, credible threats, targeted harassment.
    G5,
    /// Fraud, scams and phishing guides presented as real.
    G6,
    /// Actionable self-harm or suicide instructions or encouragement.
    G7,
    /// Legal must-remove (DMCA, court orders).
    G8,
}

impl GuardianCategory {
    pub const ALL: [GuardianCategory; 8] = [
        Self::G1, Self::G2, Self::G3, Self::G4, Self::G5, Self::G6, Self::G7, Self::G8,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::G1 => "child sexual abuse or exploitation",
            Self::G2 => "terrorism or mass-violence incitement",
            Self::G3 => "actionable serious-crime instructions",
            Self::G4 => "non-consensual intimate imagery or deepfakes",
            Self::G5 => "doxxing, threats or targeted harassment",
            Self::G6 => "fraud, scam or phishing guide",
            Self::G7 => "actionable self-harm or suicide",
            Self::G8 => "legal must-remove",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_uppercase().as_str() {
            "G1" => Self::G1,
            "G2" => Self::G2,
            "G3" => Self::G3,
            "G4" => Self::G4,
            "G5" => Self::G5,
            "G6" => Self::G6,
            "G7" => Self::G7,
            "G8" => Self::G8,
            _ => return None,
        })
    }
}

/// Guardian safety-lane verdict. Ambiguity escalates to a human rather than
/// forcing a call either way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Publish,
    Reject,
    FlagForHumanReview,
}

impl Verdict {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "publish" => Self::Publish,
            "reject" => Self::Reject,
            "flag_for_human_review" | "flag" => Self::FlagForHumanReview,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Publish => "publish",
            Self::Reject => "reject",
            Self::FlagForHumanReview => "flag_for_human_review",
        }
    }
}
