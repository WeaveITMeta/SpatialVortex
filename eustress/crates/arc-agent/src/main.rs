use anyhow::Result;
use eustress_arc_types::ArcStep;
use eustress_vortex_grid2d::Grid2D;
#[allow(unused_imports)]
use tracing::{error, info, warn};

mod policy;
mod delta_bridge;
mod toml_writer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "eustress_arc_agent=info".into()),
        )
        .init();

    // Default: EustressStream mode — in-process pub/sub, no external server
    // "standalone" mode: stdin/stdout JSON protocol (used by Python bridge)
    let mode = std::env::var("ARC_AGENT_MODE").unwrap_or_else(|_| "stream".into());

    match mode.as_str() {
        "standalone" => run_standalone_mode().await,
        // Accept "iggy" for backward compat
        "stream" | "iggy" | _ => run_stream_mode().await,
    }
}

// ─── Standalone mode: stdin/stdout JSON protocol ────────────────────────────
// This is the mode used by the Python bridge (vortex_agent.py).
// Reads observation JSON from stdin, writes action JSON to stdout.
// Background TOML writer handles debounced disk I/O.

async fn run_standalone_mode() -> Result<()> {
    use std::io::{self, BufRead, Write};

    eprintln!("[STANDALONE] Eustress arc-agent ready — reading from stdin");

    let mut policy = policy::Policy::new();
    let bg_writer = toml_writer::BackgroundTomlWriter::spawn();

    // Optional EustressStream side-channel: publish scene deltas in-process.
    #[cfg(feature = "eustress-streaming")]
    let stream_writer = {
        use eustress_common::sim_stream::SimStreamWriter;
        let stream = eustress_stream::EustressStream::new(Default::default());
        let writer = SimStreamWriter::with_stream(stream);
        eprintln!("[STANDALONE] EustressStream side-channel active");
        Some(writer)
    };
    #[cfg(not(feature = "eustress-streaming"))]
    let stream_writer: Option<()> = None;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[STANDALONE] stdin read error: {}", e);
                break;
            }
        };

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        // Parse observation JSON
        let obs: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[STANDALONE] JSON parse error: {}", e);
                let resp = serde_json::json!({
                    "action_id": 1,
                    "confidence": 0.0,
                    "reasoning": format!("parse_error: {}", e),
                });
                writeln!(stdout, "{}", resp)?;
                stdout.flush()?;
                continue;
            }
        };

        // Build ArcStep from observation
        let game_id = obs.get("game_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let step = ArcStep {
            task_id: game_id.clone(),
            step: obs.get("step").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            observation: obs.clone(),
            action_taken: None,
            terminated: obs.get("terminated").and_then(|v| v.as_bool()).unwrap_or(false),
            score: obs.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        };

        // Check for terminal state
        let state = obs.get("state").and_then(|v| v.as_str()).unwrap_or("PLAYING");
        if state == "WIN" || state == "GAME_OVER" || step.terminated {
            policy.save_knowledge();
            bg_writer.queue_game_end(game_id);

            let resp = serde_json::json!({
                "action_id": 0,
                "confidence": 1.0,
                "reasoning": format!("terminal: {}", state),
            });
            writeln!(stdout, "{}", resp)?;
            stdout.flush()?;
            continue;
        }

        // Get decision from VortexWorldModel
        let decision = policy.decide(&step);

        // Queue background TOML write (non-blocking, debounced)
        if let Some(cells) = step.frame_grid() {
            let grid = Grid2D::new(cells);
            let current_toml = policy.model().scene_mirror.to_current_toml();
            bg_writer.queue_write(game_id, grid, current_toml);
        }

        // Publish scene deltas via EustressStream (in-process, <1 µs)
        #[cfg(feature = "eustress-streaming")]
        if let Some(ref writer) = stream_writer {
            use eustress_common::stream_delta::TOPIC_SCENE_DELTAS;
            let arc_deltas = policy.last_scene_deltas();
            if !arc_deltas.is_empty() {
                let scene_deltas = delta_bridge::iggy_bridge::convert_deltas(arc_deltas);
                let producer = writer.stream().producer(TOPIC_SCENE_DELTAS);
                for delta in &scene_deltas {
                    if let Ok(bytes) = delta.to_bytes() {
                        producer.send_bytes(bytes::Bytes::from(bytes));
                    }
                }
            }
        }

        // Parse action string back to action_id + optional x,y
        let (action_id, x, y) = parse_action_string(&decision.action);

        let resp = serde_json::json!({
            "action_id": action_id,
            "x": x,
            "y": y,
            "confidence": decision.confidence,
            "reasoning": decision.reasoning,
        });

        writeln!(stdout, "{}", resp)?;
        stdout.flush()?;
    }

    eprintln!("[STANDALONE] stdin closed, exiting");
    bg_writer.shutdown();
    Ok(())
}

/// Parse action string from PolicyDecision.
/// Format: "6:x,y" for click actions, or just "1" for simple actions.
fn parse_action_string(action: &str) -> (u32, u32, u32) {
    if let Some(rest) = action.strip_prefix("6:") {
        let parts: Vec<&str> = rest.split(',').collect();
        let x = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let y = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        (6, x, y)
    } else {
        let id: u32 = action.parse().unwrap_or(1);
        (id, 0, 0)
    }
}

// ─── EustressStream mode: in-process pub/sub, no external server ─────────────
// Scene deltas published to EustressStream `scene_deltas` topic.
// TOML materializer in EustressEngine subscribes and writes debounced TOML.

#[cfg(feature = "eustress-streaming")]
async fn run_stream_mode() -> Result<()> {
    use eustress_common::stream_delta::{
        AgentAction, AgentCommand, ObservationPayload,
        TOPIC_AGENT_OBSERVATIONS,
    };
    use eustress_common::sim_record::ArcEpisodeRecord;
    use eustress_common::sim_stream::SimStreamWriter;
    use delta_bridge::iggy_bridge;
    use tokio::{signal, sync::watch, time::{interval, Duration}};
    use uuid::Uuid;
    use bytes::Bytes;

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

    tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            info!("Received shutdown signal");
            let _ = shutdown_tx.send(true);
        }
    });

    // Create in-process EustressStream — no external server needed
    let stream = eustress_stream::EustressStream::new(Default::default());
    let writer = SimStreamWriter::with_stream(stream.clone());

    // Subscribe to observations via channel
    let (obs_tx, mut obs_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    stream.subscribe_owned(TOPIC_AGENT_OBSERVATIONS, move |msg| {
        let _ = obs_tx.send(msg.data.to_vec());
    }).map_err(|e| anyhow::anyhow!("subscribe agent_observations: {e}"))?;

    info!("Arc agent ready (EustressStream mode — in-process, < 1 µs latency)");

    let now_ms = || -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    };

    let mut policy = policy::Policy::new();
    let bg_writer = toml_writer::BackgroundTomlWriter::spawn();
    let mut episode_history: Vec<ArcStep> = Vec::new();
    let mut current_task_id = String::new();
    let mut cmd_seq: u128 = 0;

    let cmd_producer = stream.producer("agent_commands");

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!("Arc agent shutting down cleanly");
                    break;
                }
            }

            Some(obs_bytes) = obs_rx.recv() => {
                let obs_payload: ObservationPayload = match rkyv::from_bytes::<_, rkyv::rancor::Error>(&obs_bytes) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Observation deserialize error: {e}");
                        continue;
                    }
                };

                if let ObservationPayload::EnvironmentState { json, step, terminated } = obs_payload {
                    let mut arc_step: ArcStep = serde_json::from_str(&json)?;
                    arc_step.step = step;
                    arc_step.terminated = terminated;

                    if current_task_id.is_empty() {
                        current_task_id = arc_step.task_id.clone();
                        info!("Starting episode for task {}", current_task_id);
                    }

                    if terminated {
                        let final_score = arc_step.score;
                        let goal_reached = final_score >= 1.0;
                        info!(task = %current_task_id, steps = episode_history.len(), score = final_score, goal_reached, "Episode complete");

                        cmd_seq += 1;
                        let cmd = AgentCommand {
                            command_id: cmd_seq,
                            action: AgentAction::EndEpisode { final_score, goal_reached },
                            script: None,
                            issued_at_ms: now_ms(),
                        };
                        if let Ok(bytes) = rkyv::to_bytes::<rkyv::rancor::Error>(&cmd) {
                            cmd_producer.send_bytes(Bytes::from(bytes.to_vec()));
                        }

                        let record = ArcEpisodeRecord {
                            episode_id: Uuid::new_v4().as_u128(),
                            task_id: current_task_id.clone(),
                            total_steps: episode_history.len() as u32,
                            goal_reached,
                            final_score,
                            human_baseline_steps: 0,
                            efficiency_ratio: 0.0,
                            actions_taken: episode_history.iter()
                                .filter_map(|s| s.action_taken.clone())
                                .collect(),
                            observations: Vec::new(),
                            duration_ms: 0,
                            completed_at_ms: now_ms(),
                            session_id: 0,
                        };
                        let _ = writer.publish_arc_episode(&record).await;

                        policy.save_knowledge();
                        bg_writer.queue_game_end(current_task_id.clone());
                        episode_history.clear();
                        current_task_id.clear();
                    } else {
                        let decision = policy.decide(&arc_step);
                        info!(step, action = %decision.action, confidence = decision.confidence, "Decided action");

                        // Queue TOML write
                        if let Some(cells) = arc_step.frame_grid() {
                            let grid = Grid2D::new(cells);
                            let current_toml = policy.model().scene_mirror.to_current_toml();
                            bg_writer.queue_write(current_task_id.clone(), grid, current_toml);
                        }

                        // Publish scene deltas (in-process, <1 µs per delta)
                        let arc_deltas = policy.last_scene_deltas();
                        if !arc_deltas.is_empty() {
                            use eustress_common::stream_delta::TOPIC_SCENE_DELTAS;
                            let scene_deltas = iggy_bridge::convert_deltas(arc_deltas);
                            let delta_producer = stream.producer(TOPIC_SCENE_DELTAS);
                            for delta in &scene_deltas {
                                if let Ok(bytes) = delta.to_bytes() {
                                    delta_producer.send_bytes(Bytes::from(bytes));
                                }
                            }
                            info!("Published {} scene deltas", scene_deltas.len());
                        }

                        cmd_seq += 1;
                        let cmd = AgentCommand {
                            command_id: cmd_seq,
                            action: AgentAction::EnvironmentAction { action: decision.action.clone(), step },
                            script: None,
                            issued_at_ms: now_ms(),
                        };
                        if let Ok(bytes) = rkyv::to_bytes::<rkyv::rancor::Error>(&cmd) {
                            cmd_producer.send_bytes(Bytes::from(bytes.to_vec()));
                        }

                        arc_step.action_taken = Some(decision.action);
                        episode_history.push(arc_step);
                    }
                }
            }
        }
    }

    bg_writer.shutdown();
    Ok(())
}

#[cfg(not(feature = "eustress-streaming"))]
async fn run_stream_mode() -> Result<()> {
    anyhow::bail!("Stream mode requires building with --features eustress-streaming")
}
