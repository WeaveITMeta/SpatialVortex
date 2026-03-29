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

    // Default: Iggy mode — full Eustress Iggy streaming loop
    // "standalone" mode: stdin/stdout JSON protocol (used by Python bridge)
    let mode = std::env::var("ARC_AGENT_MODE").unwrap_or_else(|_| "iggy".into());

    match mode.as_str() {
        "standalone" => run_standalone_mode().await,
        "iggy" | _ => run_iggy_mode().await,
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

    // Optional Iggy side-channel: publish scene deltas if server is available.
    // Falls back gracefully — TOML writer always runs regardless.
    // 2-second timeout to avoid blocking stdin loop if Iggy is down.
    #[cfg(feature = "iggy-streaming")]
    let iggy_writer = {
        use eustress_common::iggy_queue::IggyConfig;
        use eustress_common::sim_stream::SimStreamWriter;
        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            SimStreamWriter::connect(&IggyConfig::default()),
        ).await {
            Ok(Ok(w)) => {
                eprintln!("[STANDALONE] Iggy side-channel connected — publishing scene deltas");
                Some(w)
            }
            Ok(Err(e)) => {
                eprintln!("[STANDALONE] Iggy not available ({e}) — TOML-only mode");
                None
            }
            Err(_) => {
                eprintln!("[STANDALONE] Iggy connection timed out — TOML-only mode");
                None
            }
        }
    };
    #[cfg(not(feature = "iggy-streaming"))]
    let iggy_writer: Option<()> = None;

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

        // Publish scene deltas to Iggy side-channel (if connected)
        #[cfg(feature = "iggy-streaming")]
        if let Some(ref writer) = iggy_writer {
            let arc_deltas = policy.last_scene_deltas();
            if !arc_deltas.is_empty() {
                let scene_deltas = delta_bridge::iggy_bridge::convert_deltas(arc_deltas);
                if let Err(e) = writer.publish_scene_deltas(&scene_deltas).await {
                    eprintln!("[IGGY] delta publish error: {e}");
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

// ─── Iggy mode: streaming via Eustress Iggy infrastructure ──────────────────
// Binary scene deltas published to Iggy `scene_deltas` topic.
// TOML materializer in EustressEngine subscribes and writes debounced TOML.

#[cfg(feature = "iggy-streaming")]
async fn run_iggy_mode() -> Result<()> {
    use eustress_common::iggy_delta::{
        AgentAction, AgentCommand, ObservationPayload,
    };
    use eustress_common::iggy_queue::IggyConfig;
    use eustress_common::sim_record::ArcEpisodeRecord;
    use eustress_common::sim_stream::{SimStreamReader, SimStreamWriter};
    use delta_bridge::iggy_bridge;
    use tokio::{signal, sync::watch, time::{interval, Duration}};
    use uuid::Uuid;

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

    tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            info!("Received shutdown signal");
            let _ = shutdown_tx.send(true);
        }
    });

    let config = IggyConfig::default();
    let writer = SimStreamWriter::connect(&config).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut reader = SimStreamReader::connect(&config).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    info!("Arc agent connected (Iggy mode) — polling agent_observations");

    let now_ms = || -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    };

    let mut poll_ticker = interval(Duration::from_millis(10));
    let mut policy = policy::Policy::new();
    let mut episode_history: Vec<ArcStep> = Vec::new();
    let mut current_task_id = String::new();
    let mut cmd_seq: u128 = 0;

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!("Arc agent shutting down cleanly");
                    break;
                }
            }

            _ = poll_ticker.tick() => {
                match reader.poll_observation().await {
                    Ok(Some(ObservationPayload::EnvironmentState { json, step, terminated })) => {
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
                            writer.publish_command(&cmd).await
                                .map_err(|e| anyhow::anyhow!("{e}"))?;

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
                            writer.publish_arc_episode(&record).await
                                .map_err(|e| anyhow::anyhow!("{e}"))?;

                            episode_history.clear();
                            current_task_id.clear();
                        } else {
                            let decision = policy.decide(&arc_step);
                            info!(step, action = %decision.action, confidence = decision.confidence, "Decided action");

                            // Publish scene deltas to Iggy (binary, <1μs per delta)
                            let arc_deltas = policy.last_scene_deltas();
                            if !arc_deltas.is_empty() {
                                let scene_deltas = iggy_bridge::convert_deltas(arc_deltas);
                                if let Err(e) = writer.publish_scene_deltas(&scene_deltas).await {
                                    warn!("Failed to publish scene deltas: {e}");
                                } else {
                                    info!("Published {} scene deltas to Iggy", scene_deltas.len());
                                }
                            }

                            cmd_seq += 1;
                            let cmd = AgentCommand {
                                command_id: cmd_seq,
                                action: AgentAction::EnvironmentAction { action: decision.action.clone(), step },
                                script: None,
                                issued_at_ms: now_ms(),
                            };
                            writer.publish_command(&cmd).await
                                .map_err(|e| anyhow::anyhow!("{e}"))?;

                            arc_step.action_taken = Some(decision.action);
                            episode_history.push(arc_step);
                        }
                    }
                    Ok(Some(_other)) => { /* non-environment observation, skip */ }
                    Ok(None) => { /* no message yet */ }
                    Err(e) => { warn!("Poll error: {e}"); }
                }
            }
        }
    }

    Ok(())
}

#[cfg(not(feature = "iggy-streaming"))]
async fn run_iggy_mode() -> Result<()> {
    anyhow::bail!("Iggy mode requires building with --features iggy-streaming")
}
