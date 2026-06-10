//! forge-eval — Distributed benchmark fan-out via Forge orchestration
//!
//! Shards the honest benchmark suite across CPU cores: each shard is a
//! `spatialvortex-eval` process evaluating questions [offset, offset+size)
//! for one task. Forge (forge-orchestration) acts as the control plane —
//! job definition, tracking, metrics — while this binary supplies the
//! local exec layer (Forge 0.4.2's Exec driver only materializes through
//! Nomad; a local driver is upstream work for Forge 0.5).
//!
//! Usage:
//!   cargo run --release --bin forge-eval --features forge -- \
//!     --tasks mmlu,gsm8k,hellaswag,truthfulqa,humaneval
//!
//! Output: merged eval_results JSON (same schema as spatialvortex-eval)
//! plus a per-task table and the wall-clock speedup vs sequential time.

use clap::Parser;
use forge_orchestration::{Driver, ForgeBuilder, Job, Task};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

// =============================================================================
// CLI
// =============================================================================

#[derive(Parser, Debug, Clone)]
#[command(name = "forge-eval", about = "Distributed honest-benchmark runner")]
struct Args {
    /// Comma-separated tasks (must be tasks spatialvortex-eval understands)
    #[arg(long, default_value = "mmlu,gsm8k,hellaswag,truthfulqa,humaneval")]
    tasks: String,

    /// Questions per task (matches the sequential 500/task protocol)
    #[arg(long, default_value_t = 500)]
    total_limit: usize,

    /// Questions per shard process
    #[arg(long, default_value_t = 50)]
    shard_size: usize,

    /// Max concurrent shard processes (default: half the cores, capped at 24)
    #[arg(long, default_value_t = 0)]
    concurrency: usize,

    /// Path to the spatialvortex-eval binary
    #[arg(long, default_value = "")]
    eval_bin: String,

    /// Benchmark data directory (passed through to shards)
    #[arg(long, default_value = "../benchmarks/data")]
    data_dir: String,

    /// Merged output path
    #[arg(long, default_value = "eval_results_forge.json")]
    output_path: PathBuf,

    /// Directory for shard outputs (default: temp dir, cleaned on success)
    #[arg(long, default_value = "")]
    shard_dir: String,

    /// Keep per-shard JSON outputs
    #[arg(long, default_value_t = false)]
    keep_shards: bool,
}

// =============================================================================
// Shard planning
// =============================================================================

#[derive(Debug, Clone)]
struct ShardSpec {
    task: String,
    offset: usize,
    limit: usize,
    output: PathBuf,
}

/// Known dataset sizes — avoids spawning empty shards past the end
/// (the harness clamps, but an empty shard still pays process boot).
fn task_cap(task: &str, requested: usize) -> usize {
    let known_max = match task {
        "humaneval" => 164,
        _ => usize::MAX,
    };
    requested.min(known_max)
}

fn plan_shards(args: &Args, shard_dir: &std::path::Path) -> Vec<ShardSpec> {
    let mut shards = Vec::new();
    for task in args.tasks.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()) {
        let cap = task_cap(task, args.total_limit);
        let mut offset = 0;
        while offset < cap {
            let limit = args.shard_size.min(cap - offset);
            shards.push(ShardSpec {
                task: task.to_string(),
                offset,
                limit,
                output: shard_dir.join(format!("shard_{}_{}.json", task, offset)),
            });
            offset += limit;
        }
    }
    shards
}

// =============================================================================
// Shard result schema (matches spatialvortex-eval output)
// =============================================================================

#[derive(Debug, Deserialize)]
struct ShardOutput {
    task_results: Vec<ShardTaskResult>,
}

#[derive(Debug, Deserialize)]
struct ShardTaskResult {
    task: String,
    num_correct: usize,
    total: usize,
    avg_confidence: f32,
    time_secs: f64,
}

#[derive(Debug, Serialize)]
struct MergedTaskResult {
    task: String,
    accuracy: f64,
    num_correct: usize,
    total: usize,
    avg_confidence: f32,
    time_secs: f64,
    shards: usize,
}

#[derive(Debug, Serialize)]
struct MergedOutput {
    timestamp: String,
    runner: String,
    forge_job_id: String,
    shard_size: usize,
    concurrency: usize,
    task_results: Vec<MergedTaskResult>,
    overall: MergedOverall,
}

#[derive(Debug, Serialize)]
struct MergedOverall {
    total_correct: usize,
    total_questions: usize,
    overall_accuracy: f64,
    cpu_time_secs: f64,
    wall_time_secs: f64,
    speedup: f64,
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let concurrency = if args.concurrency > 0 {
        args.concurrency
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get() / 2)
            .unwrap_or(4)
            .clamp(2, 24)
    };

    let eval_bin = if args.eval_bin.is_empty() {
        format!("./target/release/spatialvortex-eval{}", std::env::consts::EXE_SUFFIX)
    } else {
        args.eval_bin.clone()
    };
    if !std::path::Path::new(&eval_bin).exists() {
        anyhow::bail!(
            "eval binary not found at {} — build it first:\n  cargo build --release --bin spatialvortex-eval --features diffusion-expert",
            eval_bin
        );
    }

    let shard_dir = if args.shard_dir.is_empty() {
        std::env::temp_dir().join(format!("forge_eval_{}", std::process::id()))
    } else {
        PathBuf::from(&args.shard_dir)
    };
    std::fs::create_dir_all(&shard_dir)?;

    let shards = plan_shards(&args, &shard_dir);

    println!("+===============================================================+");
    println!("|        FORGE-EVAL — distributed honest benchmark run          |");
    println!("+===============================================================+");
    println!("| shards: {:3}  shard_size: {:3}  concurrency: {:2}                 |",
        shards.len(), args.shard_size, concurrency);
    println!("+===============================================================+");

    // ── Forge control plane: define + submit the job ────────────────────────
    let forge = ForgeBuilder::new().with_metrics(true).build()?;
    let mut job = Job::new("honest-eval");
    for s in &shards {
        job = job.with_group(
            format!("{}-{}", s.task, s.offset),
            Task::new("eval-shard")
                .driver(Driver::Exec)
                .command(&eval_bin)
                .args(vec![
                    "--tasks".to_string(), s.task.clone(),
                    "--limit".to_string(), s.limit.to_string(),
                    "--offset".to_string(), s.offset.to_string(),
                    "--eval-only".to_string(),
                    "--skip-hf".to_string(),
                ])
                .metadata("output", s.output.display().to_string()),
        );
    }
    let job_id = forge.submit_job(job).await?;
    println!("| forge job: {}                       |", job_id);

    // ── Local exec layer (Forge 0.4.2 Exec driver needs Nomad; we run the
    //    submitted job's tasks ourselves and report back) ────────────────────
    let wall_start = Instant::now();
    let sem = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();

    for spec in shards.clone() {
        let sem = sem.clone();
        let eval_bin = eval_bin.clone();
        let data_dir = args.data_dir.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let started = Instant::now();
            let result = tokio::process::Command::new(&eval_bin)
                .args([
                    "--tasks", &spec.task,
                    "--limit", &spec.limit.to_string(),
                    "--offset", &spec.offset.to_string(),
                    "--eval-only",
                    "--skip-hf",
                    "--data-dir", &data_dir,
                    "--output-path", &spec.output.display().to_string(),
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .await;
            let ok = matches!(&result, Ok(s) if s.success());
            println!(
                "  [{}] {} offset {:3} ({} q) in {:.0}s",
                if ok { "done" } else { "FAIL" },
                spec.task, spec.offset, spec.limit,
                started.elapsed().as_secs_f64(),
            );
            (spec, ok)
        }));
    }

    let mut failed: Vec<String> = Vec::new();
    let mut completed: Vec<ShardSpec> = Vec::new();
    for h in handles {
        let (spec, ok) = h.await?;
        if ok && spec.output.exists() {
            completed.push(spec);
        } else {
            failed.push(format!("{} offset {}", spec.task, spec.offset));
        }
    }
    let wall_secs = wall_start.elapsed().as_secs_f64();

    if !failed.is_empty() {
        eprintln!("[WARN] {} shard(s) failed: {}", failed.len(), failed.join(", "));
        eprintln!("       Merged totals cover completed shards only — NOT a full run.");
    }

    // ── Merge shard outputs ──────────────────────────────────────────────────
    use std::collections::BTreeMap;
    let mut by_task: BTreeMap<String, MergedTaskResult> = BTreeMap::new();
    for spec in &completed {
        let content = std::fs::read_to_string(&spec.output)?;
        let parsed: ShardOutput = serde_json::from_str(&content)?;
        for tr in parsed.task_results {
            let entry = by_task.entry(tr.task.clone()).or_insert(MergedTaskResult {
                task: tr.task.clone(),
                accuracy: 0.0,
                num_correct: 0,
                total: 0,
                avg_confidence: 0.0,
                time_secs: 0.0,
                shards: 0,
            });
            // Confidence is averaged weighted by question count
            let new_total = entry.total + tr.total;
            if new_total > 0 {
                entry.avg_confidence = (entry.avg_confidence * entry.total as f32
                    + tr.avg_confidence * tr.total as f32) / new_total as f32;
            }
            entry.num_correct += tr.num_correct;
            entry.total = new_total;
            entry.time_secs += tr.time_secs;
            entry.shards += 1;
        }
    }
    for entry in by_task.values_mut() {
        if entry.total > 0 {
            entry.accuracy = entry.num_correct as f64 / entry.total as f64 * 100.0;
        }
    }

    let total_correct: usize = by_task.values().map(|t| t.num_correct).sum();
    let total_questions: usize = by_task.values().map(|t| t.total).sum();
    let cpu_secs: f64 = by_task.values().map(|t| t.time_secs).sum();
    let overall_acc = if total_questions > 0 {
        total_correct as f64 / total_questions as f64 * 100.0
    } else {
        0.0
    };

    println!("\n+===============================================================+");
    println!("|              FORGE-EVAL MERGED RESULTS                        |");
    println!("+----------------------+--------+----------+--------+----------+");
    println!("| Task                 | Score  | Correct  | Shards | CPU time |");
    println!("+----------------------+--------+----------+--------+----------+");
    for t in by_task.values() {
        println!("| {:20} | {:5.1}% | {:3}/{:4} | {:6} | {:7.1}s |",
            t.task, t.accuracy, t.num_correct, t.total, t.shards, t.time_secs);
    }
    println!("+----------------------+--------+----------+--------+----------+");
    println!("| OVERALL              | {:5.1}% | {:3}/{:4} |        |          |",
        overall_acc, total_correct, total_questions);
    println!("+===============================================================+");
    println!("| wall: {:6.1}s   cpu: {:7.1}s   speedup: {:4.1}x                |",
        wall_secs, cpu_secs, cpu_secs / wall_secs.max(0.001));
    println!("+===============================================================+");

    let merged = MergedOutput {
        timestamp: chrono::Utc::now().to_rfc3339(),
        runner: "forge-eval".to_string(),
        forge_job_id: job_id.clone(),
        shard_size: args.shard_size,
        concurrency,
        task_results: by_task.into_values().collect(),
        overall: MergedOverall {
            total_correct,
            total_questions,
            overall_accuracy: overall_acc,
            cpu_time_secs: cpu_secs,
            wall_time_secs: wall_secs,
            speedup: cpu_secs / wall_secs.max(0.001),
        },
    };
    std::fs::write(&args.output_path, serde_json::to_string_pretty(&merged)?)?;
    println!("\n[SAVED] Merged results -> {:?}", args.output_path);

    // Tell the control plane the job is finished, then shut down
    forge.stop_job(&job_id, true).await.ok();
    forge.shutdown().await.ok();

    if !args.keep_shards && failed.is_empty() && args.shard_dir.is_empty() {
        std::fs::remove_dir_all(&shard_dir).ok();
    }

    if failed.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("{} shard(s) failed", failed.len())
    }
}
