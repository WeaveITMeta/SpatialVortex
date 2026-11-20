//! ARC-AGI public training tasks (the local `arc_agi_training.json`), pass@1
//! exact match on the first test input. Vortex has no learned parameters for
//! ARC, so the public training set is a fair evaluation set for it; the
//! report names the split so nobody mistakes it for the private eval.

use std::path::Path;

use serde_json::Value;
use vortex_kernel::{Answer, Grid, GridTask, Response, Task};

use super::{items_from, read, sample};
use crate::{Capability, Item, Suite};

pub struct ArcAgi {
    items: Vec<Item>,
    gold: Vec<Grid>,
    source: String,
}

fn grid(v: &Value) -> Option<Grid> {
    v.as_array()?.iter().map(|row| row.as_array().map(|r| r.iter().map(|c| c.as_u64().unwrap_or(0) as u8).collect())).collect()
}

pub fn load(path: &Path, limit: usize, seed: u64) -> Result<ArcAgi, String> {
    let v: Value = serde_json::from_str(&read(path)?).map_err(|e| format!("arc-agi: {e}"))?;
    let obj = v.as_object().ok_or("arc-agi: expected an object of tasks")?;
    let mut rows = Vec::new();
    for (id, t) in obj {
        let train: Vec<(Grid, Grid)> = t["train"].as_array().map(|a| a.iter().filter_map(|p| Some((grid(&p["input"])?, grid(&p["output"])?))).collect()).unwrap_or_default();
        let Some(test) = t["test"].as_array().and_then(|a| a.first()) else { continue };
        let (Some(ti), Some(to)) = (grid(&test["input"]), grid(&test["output"])) else { continue };
        rows.push((id.clone(), Task::Grid(GridTask { train, test_input: ti }), to));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let total = rows.len();
    let rows = sample(rows, limit, seed, "arc-agi");
    let gold = rows.iter().map(|r| r.2.clone()).collect();
    let source = format!("ARC-AGI public training set, pass@1 exact, {} of {total} tasks (seed {seed})", rows.len());
    Ok(ArcAgi { items: items_from(rows.into_iter().map(|(a, b, _)| (a, b)).collect()), gold, source })
}

impl Suite for ArcAgi {
    fn id(&self) -> &str {
        "arc-agi"
    }
    fn capability(&self) -> Capability {
        Capability::Grid
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
        match &r.answer {
            Answer::Grid(g) if *g == self.gold[i] => 1.0,
            _ => 0.0,
        }
    }
}
