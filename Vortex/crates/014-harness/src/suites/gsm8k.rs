//! GSM8K test split, graded **free-form**: the final number in the answer
//! must equal the reference (after `####`). No multiple-choice crutch.

use std::path::Path;

use serde_json::Value;
use vortex_kernel::{Answer, Response, Task};

use super::{items_from, read, sample};
use crate::{Capability, Item, Suite};

pub struct Gsm8k {
    items: Vec<Item>,
    gold: Vec<f64>,
    source: String,
}

/// The last number in a string, commas and currency stripped.
pub fn last_number(s: &str) -> Option<f64> {
    // Drop thousands separators ("1,234") and currency; other commas separate.
    let chars: Vec<char> = s.chars().collect();
    let cleaned: String = chars
        .iter()
        .enumerate()
        .filter_map(|(i, &c)| {
            let thousands = c == ','
                && i > 0
                && chars[i - 1].is_ascii_digit()
                && chars.get(i + 1..i + 4).is_some_and(|d| d.iter().all(char::is_ascii_digit));
            if thousands {
                None
            } else if c == '$' || c == ',' {
                Some(' ')
            } else {
                Some(c)
            }
        })
        .collect();
    let mut found = None;
    let mut cur = String::new();
    for c in cleaned.chars().chain(std::iter::once(' ')) {
        if c.is_ascii_digit() || (c == '.' && !cur.is_empty()) || (c == '-' && cur.is_empty()) {
            cur.push(c);
        } else {
            if let Ok(v) = cur.trim_end_matches('.').parse::<f64>() {
                found = Some(v);
            }
            cur.clear();
        }
    }
    found
}

pub fn load(path: &Path, limit: usize, seed: u64) -> Result<Gsm8k, String> {
    let text = read(path)?;
    let mut rows = Vec::new();
    for (i, line) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let v: Value = serde_json::from_str(line).map_err(|e| format!("gsm8k: {e}"))?;
        let ans = v["answer"].as_str().unwrap_or("");
        let Some(g) = ans.rsplit("####").next().and_then(last_number) else { continue };
        rows.push((format!("gsm8k-{i}"), Task::Free { prompt: v["question"].as_str().unwrap_or("").to_string() }, g));
    }
    let total = rows.len();
    let rows = sample(rows, limit, seed, "gsm8k");
    let gold = rows.iter().map(|r| r.2).collect();
    let source = format!("GSM8K test, free-form exact answer, {} of {total} (seed {seed})", rows.len());
    Ok(Gsm8k { items: items_from(rows.into_iter().map(|(a, b, _)| (a, b)).collect()), gold, source })
}

impl Suite for Gsm8k {
    fn id(&self) -> &str {
        "gsm8k"
    }
    fn capability(&self) -> Capability {
        Capability::Math
    }
    fn chance(&self) -> f64 {
        0.0
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn items(&self) -> &[Item] {
        &self.items
    }
    fn grade(&self, i: usize, r: &Response) -> f64 {
        let got = match &r.answer {
            Answer::Text(t) => last_number(t),
            Answer::Numbers(v) => v.last().copied(),
            _ => None,
        };
        match got {
            Some(x) if (x - self.gold[i]).abs() < 1e-6 => 1.0,
            _ => 0.0,
        }
    }
    fn contamination_texts(&self) -> Vec<String> {
        self.items.iter().map(|it| if let Task::Free { prompt } = &it.task { prompt.clone() } else { String::new() }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::last_number;

    #[test]
    fn extracts_final_number() {
        assert_eq!(last_number("so she has $1,234.50 left."), Some(1234.5));
        assert_eq!(last_number("The answer is -7"), Some(-7.0));
        assert_eq!(last_number("none"), None);
    }
}
