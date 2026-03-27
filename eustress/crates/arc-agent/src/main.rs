use anyhow::Result;
use eustress_arc_types::ArcStep;
use eustress_common::{
    iggy_session::IggySession,
    streaming::{SimStreamReader, SimStreamWriter},
    types::{
        AgentAction, AgentCommand, ArcEpisodeRecord, ObservationPayload,
        IGGY_TOPIC_AGENT_COMMANDS, IGGY_TOPIC_AGENT_OBSERVATIONS,
    },
};
use tokio::{
    signal,
    sync::watch,
    time::{interval, Duration},
};
use tracing::{error, info, warn};
use uuid::Uuid;

mod policy;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "eustress_arc_agent=info".into()),
        )
        .init();

    // ── Shutdown channel ─────────────────────────────────────────────────────
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Spawn Ctrl-C handler.
    tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            info!("Received shutdown signal");
            let _ = shutdown_tx.send(true);
        }
    });

    run_agent_loop(shutdown_rx).await
}

async fn run_agent_loop(mut shutdown_rx: watch::Receiver<bool>) -> Result<()> {
    // ── Connect to Iggy ──────────────────────────────────────────────────────
    let session = IggySession::connect().await?;
    let mut reader =
        SimStreamReader::new(&session, IGGY_TOPIC_AGENT_OBSERVATIONS).await?;
    let writer = SimStreamWriter::new(&session).await?;

    info!("Arc agent connected — polling {IGGY_TOPIC_AGENT_OBSERVATIONS}");

    let mut poll_ticker = interval(Duration::from_millis(10));
    let mut episode_history: Vec<ArcStep> = Vec::new();
    let mut current_task_id = String::new();

    loop {
        tokio::select! {
            // ── Shutdown ─────────────────────────────────────────────────────
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!("Arc agent shutting down cleanly");
                    break;
                }
            }

            // ── Poll tick ────────────────────────────────────────────────────
            _ = poll_ticker.tick() => {
                match reader.poll_observation().await {
                    Ok(Some(payload)) => {
                        if let Err(e) = handle_observation(
                            payload,
                            &mut episode_history,
                            &mut current_task_id,
                            &writer,
                        )
                        .await
                        {
                            error!("Error handling observation: {e}");
                        }
                    }
                    Ok(None) => { /* no message yet */ }
                    Err(e) => {
                        warn!("Poll error: {e}");
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_observation(
    payload: ObservationPayload,
    episode_history: &mut Vec<ArcStep>,
    current_task_id: &mut String,
    writer: &SimStreamWriter,
) -> Result<()> {
    let ObservationPayload::EnvironmentState {
        json,
        step,
        terminated,
    } = payload;

    // ── Deserialise raw JSON into a typed ArcStep ─────────────────────────
    let mut arc_step: ArcStep = serde_json::from_str(&json)?;
    arc_step.step = step;
    arc_step.terminated = terminated;

    if current_task_id.is_empty() {
        *current_task_id = arc_step.task_id.clone();
        info!("Starting episode for task {}", current_task_id);
    }

    let available_actions = arc_step.available_actions();

    if terminated {
        // ── Episode end ───────────────────────────────────────────────────
        let final_score = arc_step.score;
        let goal_reached = final_score >= 1.0;

        info!(
            task = %current_task_id,
            steps = episode_history.len(),
            score = final_score,
            goal_reached,
            "Episode complete"
        );

        // Publish EndEpisode command.
        writer
            .publish_command(AgentCommand {
                action: AgentAction::EndEpisode {
                    final_score,
                    goal_reached,
                },
            })
            .await?;

        // Persist the full episode record to Iggy for downstream consumers.
        let record = ArcEpisodeRecord {
            episode_id: Uuid::new_v4().to_string(),
            task_id: current_task_id.clone(),
            steps: episode_history.len() as u32,
            goal_reached,
            final_score,
            history: episode_history.clone(),
        };
        writer.publish_arc_episode(record).await?;

        // Reset for the next episode.
        episode_history.clear();
        current_task_id.clear();
    } else {
        // ── Mid-episode step ──────────────────────────────────────────────
        let decision = policy::decide(episode_history, &available_actions);

        info!(
            step,
            action = %decision.action,
            confidence = decision.confidence,
            "Decided action"
        );

        writer
            .publish_command(AgentCommand {
                action: AgentAction::EnvironmentAction {
                    action: decision.action.clone(),
                    step,
                },
            })
            .await?;

        // Record what was actually taken.
        arc_step.action_taken = Some(decision.action);
        episode_history.push(arc_step);
    }

    Ok(())
}
