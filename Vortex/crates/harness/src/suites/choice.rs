//! Multiple-choice suites: HellaSwag (validation), AI2 ARC (test splits),
//! MMLU (test split). Gold indices stay in the suite.

use std::path::Path;

use serde_json::Value;
use vortex_kernel::{Answer, Response, Task};

use super::{items_from, read, sample};
use crate::{Capability, Item, Suite};

pub struct ChoiceSuite {
    id: String,
    capability: Capability,
    source: String,
    items: Vec<Item>,
    gold: Vec<usize>,
    chance: f64,
}

impl Suite for ChoiceSuite {
    fn id(&self) -> &str {
        &self.id
    }
    fn capability(&self) -> Capability {
        self.capability
    }
    fn chance(&self) -> f64 {
        self.chance
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn items(&self) -> &[Item] {
        &self.items
    }
    fn grade(&self, i: usize, r: &Response) -> f64 {
        match r.answer {
            Answer::Choice(k) if k == self.gold[i] => 1.0,
            _ => 0.0,
        }
    }
    fn contamination_texts(&self) -> Vec<String> {
        self.items
            .iter()
            .zip(&self.gold)
            .map(|(it, g)| match &it.task {
                Task::Choice { context, question, choices } => format!("{context} {question} {}", choices[*g]),
                _ => String::new(),
            })
            .collect()
    }
}

impl ChoiceSuite {
    fn build(id: &str, capability: Capability, source: String, rows: Vec<(String, Task, usize)>) -> Self {
        let n_choices: f64 = rows
            .iter()
            .map(|(_, t, _)| match t {
                Task::Choice { choices, .. } => 1.0 / choices.len().max(1) as f64,
                _ => 0.0,
            })
            .sum();
        let chance = n_choices / rows.len().max(1) as f64;
        let gold = rows.iter().map(|(_, _, g)| *g).collect();
        let items = items_from(rows.into_iter().map(|(id, t, _)| (id, t)).collect());
        Self { id: id.into(), capability, source, items, gold, chance }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

pub fn hellaswag(path: &Path, limit: usize, seed: u64) -> Result<ChoiceSuite, String> {
    let text = read(path)?;
    let mut rows = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let v: Value = serde_json::from_str(line).map_err(|e| format!("hellaswag: {e}"))?;
        let endings: Vec<String> = v["endings"].as_array().ok_or("hellaswag: no endings")?.iter().filter_map(|e| e.as_str().map(String::from)).collect();
        let label = v["label"].as_u64().or_else(|| v["label"].as_str().and_then(|s| s.parse().ok())).ok_or("hellaswag: no label")? as usize;
        rows.push((
            format!("hellaswag-{}", v["ind"]),
            Task::Choice { context: v["ctx"].as_str().unwrap_or("").to_string(), question: String::new(), choices: endings },
            label,
        ));
    }
    let total = rows.len();
    let rows = sample(rows, limit, seed, "hellaswag");
    Ok(ChoiceSuite::build("hellaswag", Capability::Language, format!("HellaSwag validation, {} of {total} sampled (seed {seed})", rows.len()), rows))
}

pub fn ai2_arc(id: &str, path: &Path, limit: usize, seed: u64) -> Result<ChoiceSuite, String> {
    let rows = load_ai2_arc(path)?;
    let total = rows.len();
    let rows = sample(rows, limit, seed, id);
    let file = path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(ChoiceSuite::build(id, Capability::Knowledge, format!("AI2 {file}, {} of {total} sampled (seed {seed})", rows.len()), rows))
}

/// Unsampled AI2 ARC rows: used directly for dev splits.
pub fn ai2_arc_all(id: &str, path: &Path) -> Result<ChoiceSuite, String> {
    let rows = load_ai2_arc(path)?;
    let file = path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(ChoiceSuite::build(id, Capability::Knowledge, format!("AI2 {file}, all {}", rows.len()), rows))
}

fn load_ai2_arc(path: &Path) -> Result<Vec<(String, Task, usize)>, String> {
    let text = read(path)?;
    let mut rows = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let v: Value = serde_json::from_str(line).map_err(|e| format!("arc: {e}"))?;
        let choices = v["question"]["choices"].as_array().ok_or("arc: no choices")?;
        let labels: Vec<String> = choices.iter().map(|c| c["label"].as_str().unwrap_or("").to_string()).collect();
        let texts: Vec<String> = choices.iter().map(|c| c["text"].as_str().unwrap_or("").to_string()).collect();
        let key = v["answerKey"].as_str().unwrap_or("");
        let Some(gold) = labels.iter().position(|l| l == key) else { continue };
        rows.push((
            v["id"].as_str().unwrap_or("").to_string(),
            Task::Choice { context: String::new(), question: v["question"]["stem"].as_str().unwrap_or("").to_string(), choices: texts },
            gold,
        ));
    }
    Ok(rows)
}

/// Minimal RFC-4180 CSV: quoted fields, doubled quotes, embedded newlines.
fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' => quoted = true,
                ',' => row.push(std::mem::take(&mut field)),
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                }
                '\r' => {}
                _ => field.push(c),
            }
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

pub fn mmlu(dir: &Path, limit: usize, seed: u64) -> Result<ChoiceSuite, String> {
    let mut files: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "csv")).collect();
    files.sort();
    let mut rows = Vec::new();
    for f in &files {
        let subject = f.file_stem().map(|s| s.to_string_lossy().trim_end_matches("_test").replace('_', " ")).unwrap_or_default();
        for (i, r) in parse_csv(&read(f)?).into_iter().enumerate() {
            if r.len() < 6 {
                continue;
            }
            let gold = match r[5].trim() {
                "A" => 0,
                "B" => 1,
                "C" => 2,
                "D" => 3,
                _ => continue,
            };
            rows.push((
                format!("mmlu-{}-{i}", subject.replace(' ', "_")),
                Task::Choice { context: format!("The following is a question about {subject}."), question: r[0].clone(), choices: r[1..5].to_vec() },
                gold,
            ));
        }
    }
    let total = rows.len();
    let rows = sample(rows, limit, seed, "mmlu");
    Ok(ChoiceSuite::build("mmlu", Capability::Knowledge, format!("MMLU test, {} of {total} across {} subjects (seed {seed})", rows.len(), files.len()), rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_handles_quotes_and_commas() {
        let rows = parse_csv("a,\"b, c\",\"say \"\"hi\"\"\"\n1,2,3\n");
        assert_eq!(rows[0], vec!["a", "b, c", "say \"hi\""]);
        assert_eq!(rows[1], vec!["1", "2", "3"]);
    }
}
