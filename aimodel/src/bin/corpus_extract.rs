//! Transition corpus extractor for the grid-diffusion world model.
//!
//! Scans an ARC-AGI-3 recordings directory for *.jsonl session files and emits
//! one JSON object per consecutive (grid_t, action, grid_t+1) transition.
//!
//! Recording line schema (one per executed action):
//!   {"timestamp": ..., "data": {"game_id", "frame": [[[u8]]], "state",
//!    "levels_completed", "action_input": {"id", "data": {"x"?, "y"?}}, ...}}
//!
//! `frame` is a list of animation frames; the last one is the settled grid.
//! `action_input` on record N is the action that produced record N's frame, so
//! a transition pairs record N-1's grid with record N's action and grid.
//!
//! Usage: corpus_extract [--recordings DIR] [--out FILE]

use serde::Serialize;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const DEFAULT_RECORDINGS: &str = "E:/Libraries/SpatialVortex/ARC-AGI-3-Agents/recordings";
const DEFAULT_OUT: &str = "./corpus.jsonl";

#[derive(Serialize)]
struct Transition {
    game: String,
    step: usize,
    action: String,
    grid_before: Vec<Vec<u8>>,
    grid_after: Vec<Vec<u8>>,
    levels: i64,
    win: bool,
    death: bool,
}

/// One parsed recording line.
struct Record {
    game_id: String,
    grid: Vec<Vec<u8>>,
    action: String,
    levels: i64,
    state: String,
    full_reset: bool,
}

#[derive(Default)]
struct Stats {
    files: usize,
    transitions: usize,
    wins: usize,
    deaths: usize,
    malformed: usize,
}

fn main() {
    let mut recordings = PathBuf::from(DEFAULT_RECORDINGS);
    let mut out_path = PathBuf::from(DEFAULT_OUT);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--recordings" if i + 1 < args.len() => {
                recordings = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--out" if i + 1 < args.len() => {
                out_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                eprintln!("Usage: corpus_extract [--recordings DIR] [--out FILE]");
                std::process::exit(2);
            }
        }
    }

    let mut stats = Stats::default();
    let out_file = File::create(&out_path).unwrap_or_else(|e| {
        eprintln!("Cannot create {}: {e}", out_path.display());
        std::process::exit(1);
    });
    let mut out = BufWriter::new(out_file);

    let entries = std::fs::read_dir(&recordings).unwrap_or_else(|e| {
        eprintln!("Cannot read {}: {e}", recordings.display());
        std::process::exit(1);
    });
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |x| x == "jsonl"))
        .collect();
    paths.sort();

    for path in &paths {
        stats.files += 1;
        if let Err(e) = extract_file(path, &mut out, &mut stats) {
            eprintln!("Skipping {}: {e}", path.display());
        }
    }
    out.flush().expect("flush corpus output");

    println!("Files scanned:         {}", stats.files);
    println!("Transitions extracted: {}", stats.transitions);
    println!("Wins:                  {}", stats.wins);
    println!("Deaths:                {}", stats.deaths);
    println!("Malformed lines:       {}", stats.malformed);
    println!("Corpus written to {}", out_path.display());
}

fn extract_file(path: &Path, out: &mut impl Write, stats: &mut Stats) -> std::io::Result<()> {
    let reader = BufReader::new(File::open(path)?);
    let mut prev: Option<Record> = None;
    let mut step = 0usize;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let rec = match parse_record(&line) {
            Some(r) => r,
            None => {
                stats.malformed += 1;
                continue;
            }
        };
        if let Some(p) = &prev {
            // A full reset starts a fresh episode; the before/after pair is
            // not a causal consequence of the action, so don't emit it.
            if !rec.full_reset {
                let win = rec.levels > p.levels;
                let death = rec.state == "GAME_OVER";
                let t = Transition {
                    game: rec.game_id.clone(),
                    step,
                    action: rec.action.clone(),
                    grid_before: p.grid.clone(),
                    grid_after: rec.grid.clone(),
                    levels: rec.levels,
                    win,
                    death,
                };
                writeln!(out, "{}", serde_json::to_string(&t).expect("serialize transition"))?;
                stats.transitions += 1;
                if win {
                    stats.wins += 1;
                }
                if death {
                    stats.deaths += 1;
                }
            }
        }
        step += 1;
        prev = Some(rec);
    }
    Ok(())
}

/// Parses one recording line; None for malformed or frameless records.
fn parse_record(line: &str) -> Option<Record> {
    let v: Value = serde_json::from_str(line).ok()?;
    let data = v.get("data")?;
    let grid = last_grid(data.get("frame")?)?;
    let action_input = data.get("action_input")?;
    Some(Record {
        game_id: data.get("game_id")?.as_str()?.to_string(),
        grid,
        action: action_name(action_input),
        levels: data.get("levels_completed").and_then(Value::as_i64).unwrap_or(0),
        state: data
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        full_reset: data
            .get("full_reset")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

/// The last animation frame is the settled grid for the step.
fn last_grid(frame: &Value) -> Option<Vec<Vec<u8>>> {
    let last = frame.as_array()?.last()?;
    let mut grid = Vec::new();
    for row in last.as_array()? {
        let row: Option<Vec<u8>> = row
            .as_array()?
            .iter()
            .map(|c| c.as_u64().map(|n| n.min(255) as u8))
            .collect();
        grid.push(row?);
    }
    if grid.is_empty() {
        None
    } else {
        Some(grid)
    }
}

/// "RESET" | "ACTION{n}" | "ACTION6[x,y]" for coordinate actions.
fn action_name(action_input: &Value) -> String {
    let id = action_input.get("id").and_then(Value::as_u64).unwrap_or(0);
    if id == 0 {
        return "RESET".to_string();
    }
    let coords = action_input.get("data").and_then(|d| {
        Some((
            d.get("x")?.as_u64()?,
            d.get("y")?.as_u64()?,
        ))
    });
    match coords {
        Some((x, y)) => format!("ACTION{id}[{x},{y}]"),
        None => format!("ACTION{id}"),
    }
}
