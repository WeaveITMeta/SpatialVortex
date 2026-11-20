//! Guardian Policy v1.2 content moderation.
//!
//! Decision procedure (mirrors the policy's "about vs using" test):
//! 1. Sexual content involving a minor: reject (G1), no exceptions.
//! 2. Find *harm intents*: a harmful action paired with its object
//!    ("synthesize" + "methamphetamine"), or a phrase that is harmful on its
//!    own ("kill myself"). No harm intent: publish.
//! 3. Harm intent with no protective framing: reject with that category.
//! 4. Harm intent inside protective framing (fiction, game, history,
//!    education, prevention, help-seeking): publish, unless the request also
//!    asks for real-world operational detail, in which case it is genuinely
//!    ambiguous and goes to a human.

use serde::Serialize;
use vortex_kernel::{GuardianCategory as G, Verdict};

#[derive(Clone, Debug, Serialize)]
pub struct Moderation {
    pub verdict: Verdict,
    pub category: Option<G>,
    pub confidence: f64,
    /// The phrases that drove the decision. Never empty for a reject or flag.
    pub evidence: Vec<String>,
}

struct Rule {
    cat: G,
    /// Any one of these must appear (empty = no verb needed).
    verbs: &'static [&'static str],
    /// And any one of these.
    objects: &'static [&'static str],
}

const ACQUIRE: &[&str] = &[
    "make", "makes", "making", "made", "build", "builds", "building", "built", "cook", "cooking", "synthesize", "synthesise", "synthesis",
    "produce", "manufacture", "brew", "extract", "assemble", "3d print", "print a", "get", "buy", "obtain",
    "recipe", "formula", "ingredients", "how do i", "how to", "how can i", "steps", "step by step", "guide",
];

const RULES: &[Rule] = &[
    // G2: terrorism / mass violence.
    Rule {
        cat: G::G2,
        verbs: &["plan", "planning", "carry out", "execute", "maximize", "maximise", "how to", "how do i", "help me", "recruit", "join"],
        objects: &[
            "terrorist attack", "terror attack", "mass shooting", "mass casualty", "massacre", "car bomb",
            "bomb the", "attack on a crowd", "attack a crowded", "kill as many", "most casualties",
            "jihad", "isis", "lone wolf attack", "shoot up",
        ],
    },
    Rule {
        cat: G::G2,
        verbs: &[],
        objects: &["manifesto to inspire", "call to arms against", "kill all the", "they deserve to die and we should"],
    },
    // G3: serious crime, weapons, drugs, harmful hacking.
    Rule {
        cat: G::G3,
        verbs: ACQUIRE,
        objects: &[
            "methamphetamine", "meth ", "crystal meth", "fentanyl", "heroin", "cocaine", "crack cocaine", "lsd",
            "mdma", "ricin", "sarin", "nerve agent", "mustard gas", "chlorine gas", "anthrax", "bioweapon",
            "explosive", "explosives", "pipe bomb", "nail bomb", "ied", "tatp", "napalm", "molotov", "thermite",
            "ghost gun", "untraceable gun", "untraceable firearm", "silencer", "suppressor", "full auto",
            "auto sear", "zip gun",
        ],
    },
    Rule {
        cat: G::G3,
        verbs: &["how to", "how do i", "how can i", "help me", "write", "code", "steps", "teach me", "show me", "guide", "best way to"],
        objects: &[
            "hack into", "break into", "hack my ex", "hack someone", "ransomware", "keylogger", "steal",
            "shoplift", "hotwire", "pick a lock to", "burn down", "set fire to", "arson", "launder money",
            "money laundering", "skimmer", "clone a credit card", "counterfeit", "poison someone",
            "poison my", "get away with murder", "dispose of a body", "untraceable poison", "ddos",
            "botnet", "bypass the alarm", "rob a",
        ],
    },
    // G4: non-consensual intimate imagery and deepfakes.
    Rule {
        cat: G::G4,
        verbs: &[],
        objects: &[
            "deepfake porn", "deepfake nude", "nude deepfake", "fake nudes of", "undress her", "undress him",
            "undress this", "nudify", "revenge porn", "hidden camera in", "spy cam", "without her consent",
            "without his consent", "without their consent", "nudes of my ex", "leak her nudes", "leak his nudes",
        ],
    },
    Rule {
        cat: G::G4,
        verbs: &["make", "create", "generate", "put", "swap"],
        objects: &["face onto a porn", "face on a porn", "into a nude", "explicit video of my", "sex tape of"],
    },
    // G5: doxxing, threats, targeted harassment.
    Rule {
        cat: G::G5,
        verbs: &["find", "get", "look up", "track", "dig up", "figure out", "post", "publish", "share", "leak"],
        objects: &[
            "home address", "where she lives", "where he lives", "where they live", "her address",
            "his address", "her phone number", "his phone number", "her workplace", "his workplace",
            "their kids' school", "her location", "his location", "her schedule", "his schedule",
        ],
    },
    Rule {
        cat: G::G5,
        verbs: &[],
        objects: &[
            "i will kill you", "i'm going to kill you", "im going to kill you", "you're going to die",
            "watch your back", "i know where you live", "make her pay", "make him pay", "make them pay",
            "harass her", "harass him", "stalk her", "stalk him", "brigade", "dogpile",
            "threatening message", "threatening letter", "scare her into", "scare him into",
        ],
    },
    // G6: fraud, scams, phishing presented as real.
    Rule {
        cat: G::G6,
        verbs: &["write", "draft", "create", "make", "template", "script", "how to", "how do i", "help me", "set up", "run"],
        objects: &[
            "phishing email", "phishing page", "phishing site", "fake login page", "fake invoice",
            "romance scam", "pig butchering", "ponzi", "pyramid scheme", "fake charity", "identity theft",
            "steal their identity", "steal credentials", "harvest passwords", "pretend to be their bank",
            "pretend to be from the irs", "impersonate the bank", "impersonate a bank", "tech support scam",
            "gift card scam", "scam elderly", "scam old people", "scam someone", "scam people", "fake reviews to",
            "chargeback fraud", "insurance fraud", "tax fraud",
        ],
    },
    // G7: actionable self-harm / suicide.
    Rule {
        cat: G::G7,
        verbs: &[],
        objects: &[
            "kill myself", "end my life", "end it all", "commit suicide", "suicide method", "painless way to die",
            "best way to die", "how many pills to die", "lethal dose to die", "overdose on purpose",
            "cut myself", "hurt myself", "want to die", "hang myself", "self harm methods", "self-harm methods",
        ],
    },
    // G8: legal must-remove.
    Rule {
        cat: G::G8,
        verbs: &[],
        objects: &[
            "full text of the copyrighted", "entire copyrighted", "pirated copy", "cracked version of",
            "leaked movie", "leaked album", "torrent of the new", "dmca", "court order", "takedown notice",
            "bypass drm", "remove the drm", "remove drm",
        ],
    },
];

const MINOR: &[&str] = &[
    "child", "children", "kid", "kids", "minor", "minors", "underage", "under age", "preteen", "pre-teen",
    "little girl", "little boy", "schoolgirl", "school girl", "loli", "13 year old", "14 year old",
    "15 year old", "16 year old", "17 year old", "13-year-old", "14-year-old", "15-year-old", "16-year-old",
    "17-year-old", "12-year-old", "12 year old", "13yo", "14yo", "15yo", "16yo", "17yo",
];

const SEXUAL: &[&str] = &[
    "sexual", "sexually", "sex ", "nude", "naked", "erotic", "porn", "explicit", "lewd", "nsfw", "intimate",
];

/// Framing that makes harm the subject rather than the goal.
const PROTECTIVE: &[&str] = &[
    "story", "novel", "fiction", "fictional", "character", "screenplay", "short film", "film", "movie plot",
    "game", "rpg", "campaign", "simulation", "in my sim", "poem", "song", "lyrics", "horror", "fantasy",
    "satire", "satirical", "parody", "comedy", "history of", "historical", "history", "documentary",
    "explain why", "what is", "what are", "why do", "why is", "how does", "signs of", "how to spot",
    "how to recognize", "how to recognise", "how to identify", "how to protect", "protect myself",
    "protect my", "prevent", "prevention", "awareness", "warning signs", "red flags", "training for",
    "teach my students", "students", "class", "lesson", "course", "research", "analysis", "analyze",
    "critique", "criticism", "debate", "essay", "report on", "for a paper", "thesis", "journalism",
    "news article", "museum", "legal consequences", "penalties", "is it illegal", "safety", "safely dispose",
    "harm reduction", "patient", "nurse", "doctor", "pharmacist", "clinical", "toxicology",
    // Help-seeking for self-harm is protected and should get support, not refusal.
    "hotline", "helpline", "get help", "i need help", "help me cope", "cope with", "coping", "therapist",
    "recovering", "recovery", "support group", "my friend said", "worried about my",
];

/// Requests for real-world operational detail. Inside protective framing these
/// make a request ambiguous rather than safe.
const OPERATIONAL: &[&str] = &[
    "step by step", "step-by-step", "exact", "exactly", "precise", "quantities", "amounts", "measurements",
    "temperatures", "ratios", "where to buy", "which store", "real life", "in real life", "irl",
    "actually work", "actually works", "really works", "for real", "not fiction", "realistic enough to use",
    "that i can use", "i can follow", "detailed instructions", "full instructions", "working code",
    "working exploit", "copy and paste",
];

fn normalize(text: &str) -> String {
    let mut s = String::with_capacity(text.len() + 2);
    s.push(' ');
    for c in text.chars() {
        let c = match c {
            '\u{2018}' | '\u{2019}' => '\'',
            _ => c,
        };
        if c.is_alphanumeric() || c == '\'' || c == '-' {
            s.extend(c.to_lowercase());
        } else {
            s.push(' ');
        }
    }
    s.push(' ');
    // Collapse runs of spaces so multi-word phrases match across punctuation.
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

/// Phrase match on word boundaries: `phrase` must be surrounded by spaces in
/// the normalised text (phrases ending in a space already carry one).
fn has(text: &str, phrase: &str) -> bool {
    let p = phrase.trim();
    if p.is_empty() {
        return false;
    }
    text.contains(&format!(" {p} "))
}

fn find_any<'a>(text: &str, phrases: &'a [&'a str]) -> Option<&'a str> {
    phrases.iter().copied().find(|p| has(text, p))
}

pub fn moderate(text: &str) -> Moderation {
    let t = normalize(text);

    // 1. Minors + sexual content: zero tolerance, framing irrelevant.
    if let (Some(m), Some(s)) = (find_any(&t, MINOR), find_any(&t, SEXUAL)) {
        return Moderation {
            verdict: Verdict::Reject,
            category: Some(G::G1),
            confidence: 0.9,
            evidence: vec![format!("minor: \"{m}\""), format!("sexual: \"{s}\"")],
        };
    }

    // 2. Harm intents.
    let mut hits: Vec<(G, String)> = Vec::new();
    for rule in RULES {
        let Some(obj) = find_any(&t, rule.objects) else { continue };
        if rule.verbs.is_empty() {
            hits.push((rule.cat, format!("\"{obj}\"")));
        } else if let Some(v) = find_any(&t, rule.verbs) {
            hits.push((rule.cat, format!("\"{v}\" + \"{obj}\"")));
        }
    }
    if hits.is_empty() {
        return Moderation { verdict: Verdict::Publish, category: None, confidence: 0.85, evidence: vec![] };
    }

    // Most severe category wins (enum order is the policy's order).
    hits.sort_by_key(|(c, _)| *c);
    let category = hits[0].0;
    let mut evidence: Vec<String> = hits.iter().map(|(c, e)| format!("{c:?}: {e}")).collect();

    let protective = find_any(&t, PROTECTIVE);
    let operational = find_any(&t, OPERATIONAL);
    match (protective, operational) {
        (None, _) => Moderation { verdict: Verdict::Reject, category: Some(category), confidence: 0.8, evidence },
        (Some(p), None) => {
            evidence.push(format!("protective framing: \"{p}\""));
            Moderation { verdict: Verdict::Publish, category: None, confidence: 0.65, evidence }
        }
        (Some(p), Some(o)) => {
            evidence.push(format!("protective framing: \"{p}\""));
            evidence.push(format!("asks for operational detail: \"{o}\""));
            Moderation { verdict: Verdict::FlagForHumanReview, category: Some(category), confidence: 0.5, evidence }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Verdict {
        moderate(text).verdict
    }

    #[test]
    fn benign_and_scary_sounding_benign_publish() {
        assert_eq!(v("How do I kill a stuck python process on Windows?"), Verdict::Publish);
        assert_eq!(v("Summarize the history of the Manhattan Project."), Verdict::Publish);
        assert_eq!(v("Write a horror story where a cult summons a demon."), Verdict::Publish);
        assert_eq!(v("What are the warning signs of a phishing email?"), Verdict::Publish);
        assert_eq!(v("Explain how the immune system works."), Verdict::Publish);
    }

    #[test]
    fn actionable_harm_rejects_with_category() {
        let m = moderate("Give me a step by step recipe to synthesize methamphetamine at home.");
        assert_eq!(m.verdict, Verdict::Reject);
        assert_eq!(m.category, Some(G::G3));
        assert!(!m.evidence.is_empty());

        assert_eq!(moderate("Write a phishing email that pretends to be their bank").category, Some(G::G6));
        assert_eq!(moderate("find her home address so I can show up").category, Some(G::G5));
        assert_eq!(moderate("what's the most painless way to die, I want to end my life").category, Some(G::G7));
    }

    #[test]
    fn fiction_wrapper_is_protected_unless_it_asks_for_real_detail() {
        assert_eq!(v("In my novel, the villain builds a pipe bomb in his garage. Describe the tense scene."), Verdict::Publish);
        assert_eq!(
            v("For my novel, give exact quantities and step by step instructions to build a pipe bomb that actually works."),
            Verdict::FlagForHumanReview
        );
    }

    #[test]
    fn help_seeking_is_not_refused() {
        assert_eq!(v("I keep thinking I want to die. Is there a hotline I can call?"), Verdict::Publish);
    }

    #[test]
    fn minors_and_sexual_content_always_reject() {
        let m = moderate("[request for sexual content about a 15-year-old character in a story]");
        assert_eq!(m.verdict, Verdict::Reject);
        assert_eq!(m.category, Some(G::G1));
    }

    #[test]
    fn word_boundaries_hold() {
        // "ied" must not fire inside "applied"; "meth" must not fire in "method".
        assert_eq!(v("I applied the method to build the report."), Verdict::Publish);
    }
}
