//! HumanEval, graded by **executing** the completion against the dataset's
//! hidden unit tests in Python with a timeout. Generated code runs in a
//! fresh temp directory, never in the repository. Abstentions are scored
//! without running anything.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use vortex_kernel::{Answer, Response, Task};

use super::{items_from, read, sample};
use crate::{Capability, Item, Suite};

pub struct HumanEval {
    items: Vec<Item>,
    /// (full prompt, test code, entry point)
    hidden: Vec<(String, String, String)>,
    source: String,
    pub timeout: Duration,
}

pub fn humaneval(path: &Path, limit: usize, seed: u64) -> Result<HumanEval, String> {
    let text = read(path)?;
    let mut rows = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let v: Value = serde_json::from_str(line).map_err(|e| format!("humaneval: {e}"))?;
        let prompt = v["prompt"].as_str().unwrap_or("").to_string();
        let entry = v["entry_point"].as_str().unwrap_or("").to_string();
        rows.push((
            v["task_id"].as_str().unwrap_or("").to_string(),
            Task::Code { prompt: prompt.clone(), entry_point: entry.clone() },
            (prompt, v["test"].as_str().unwrap_or("").to_string(), entry),
        ));
    }
    let total = rows.len();
    let rows = sample(rows, limit, seed, "humaneval");
    let source = format!("HumanEval, pass@1 by execution, {} of {total} (seed {seed})", rows.len());
    let hidden = rows.iter().map(|r| r.2.clone()).collect();
    Ok(HumanEval { items: items_from(rows.into_iter().map(|(a, b, _)| (a, b)).collect()), hidden, source, timeout: Duration::from_secs(10) })
}

fn python() -> &'static str {
    if cfg!(windows) { "python" } else { "python3" }
}

/// Run `program` with Python; true when it exits 0 within the timeout.
pub fn run_python(program: &str, timeout: Duration) -> bool {
    let dir = std::env::temp_dir().join(format!("vortex-humaneval-{}-{}", std::process::id(), vortex_kernel::fnv1a64(program.as_bytes())));
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let file = dir.join("candidate.py");
    if std::fs::write(&file, program).is_err() {
        return false;
    }
    let Ok(mut child) = Command::new(python()).arg(&file).current_dir(&dir).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn() else {
        return false;
    };
    let t0 = Instant::now();
    let ok = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st.success(),
            Ok(None) if t0.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                break false;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(_) => break false,
        }
    };
    if let Some(mut e) = child.stderr.take() {
        let mut s = String::new();
        let _ = e.read_to_string(&mut s);
    }
    let _ = std::fs::remove_dir_all(&dir);
    ok
}

impl Suite for HumanEval {
    fn id(&self) -> &str {
        "humaneval"
    }
    fn capability(&self) -> Capability {
        Capability::Code
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
        let Answer::Text(completion) = &r.answer else { return 0.0 };
        let (prompt, test, entry) = &self.hidden[i];
        let program = format!("{prompt}{completion}\n\n{test}\n\ncheck({entry})\n");
        if run_python(&program, self.timeout) { 1.0 } else { 0.0 }
    }
    fn contamination_texts(&self) -> Vec<String> {
        self.hidden.iter().map(|h| h.0.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "needs Python on PATH"]
    fn executes_and_times_out() {
        assert!(run_python("assert 1 + 1 == 2\n", Duration::from_secs(10)));
        assert!(!run_python("assert 1 + 1 == 3\n", Duration::from_secs(10)));
        assert!(!run_python("while True: pass\n", Duration::from_millis(500)));
    }
}
