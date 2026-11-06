use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    pkg::{
        errors::AppError,
        realtime::RealtimeHub,
        response::BaseResponse,
        validation::ValidatedJson,
    },
};
use super::{
    dto::{AgentToControllerMessage, ControllerToAgentMessage, RegisterAgentRequest},
    service::AgentService,
};

#[derive(Clone)]
pub struct AgentContext {
    pub service: Arc<AgentService>,
    pub realtime_hub: Arc<RealtimeHub>,
    pub pool: sqlx::PgPool,
}

pub async fn register_agent_handler(
    State(ctx): State<AgentContext>,
    ValidatedJson(payload): ValidatedJson<RegisterAgentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let agent = ctx.service.register_agent(payload).await?;
    Ok(BaseResponse::created(agent, "Agent registered successfully"))
}

pub async fn list_agents_handler(
    State(ctx): State<AgentContext>,
) -> Result<impl IntoResponse, AppError> {
    let agents = ctx.service.list_agents().await?;
    Ok(BaseResponse::success(agents, "Agents retrieved successfully"))
}

pub async fn agent_ws_handler(
    ws: WebSocketUpgrade,
    State(ctx): State<AgentContext>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_agent_socket(socket, ctx.service, ctx.realtime_hub, ctx.pool))
}

async fn handle_agent_socket(
    socket: WebSocket,
    agent_service: Arc<AgentService>,
    realtime_hub: Arc<RealtimeHub>,
    pool: sqlx::PgPool,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (agent_tx, mut agent_rx) = mpsc::unbounded_channel::<ControllerToAgentMessage>();

    let mut authenticated_agent_id: Option<Uuid> = None;

    let send_task = tokio::spawn(async move {
        while let Some(msg) = agent_rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&msg) {
                if ws_tx.send(Message::Text(json_str)).await.is_err() {
                    break;
                }
            }
        }
    });

    while let Some(Ok(msg)) = ws_rx.next().await {
        if let Message::Text(text) = msg {
            let parsed: Result<AgentToControllerMessage, _> = serde_json::from_str(&text);
            match parsed {
                Ok(AgentToControllerMessage::Register { token, name, tags: _, os_info }) => {
                    match agent_service.authenticate_by_token(&token).await {
                        Ok(Some(agent)) => {
                            authenticated_agent_id = Some(agent.id);
                            let socket_id = Uuid::new_v4().to_string();
                            let _ = agent_service.mark_agent_online(agent.id, &socket_id, os_info, agent_tx.clone()).await;

                            let _ = agent_tx.send(ControllerToAgentMessage::Registered {
                                agent_id: agent.id,
                            });
                            info!(agent_id = %agent.id, agent_name = %name, "Agent authenticated successfully");
                        }
                        _ => {
                            warn!("Agent authentication failed with invalid token");
                            let _ = agent_tx.send(ControllerToAgentMessage::Error {
                                message: "Invalid agent token".to_string(),
                            });
                            break;
                        }
                    }
                }
                Ok(AgentToControllerMessage::Heartbeat) => {
                    if let Some(agent_id) = authenticated_agent_id {
                        let _ = agent_service.heartbeat(agent_id).await;
                    }
                }
                Ok(AgentToControllerMessage::JobAccepted { job_id }) => {
                    info!(job_id = %job_id, "Agent accepted job");
                    let _ = sqlx::query("UPDATE jobs SET status = 'running', started_at = NOW() WHERE id = $1")
                        .bind(job_id)
                        .execute(&pool)
                        .await;
                }
                Ok(AgentToControllerMessage::LogStream { job_id, step_index, stream, line }) => {
                    let _ = sqlx::query(
                        "INSERT INTO job_logs (job_id, step_index, stream, line) VALUES ($1, $2, $3, $4)"
                    )
                    .bind(job_id)
                    .bind(step_index as i32)
                    .bind(&stream)
                    .bind(&line)
                    .execute(&pool)
                    .await;

                    let payload = serde_json::json!({
                        "topic": format!("job_logs:{}", job_id),
                        "job_id": job_id,
                        "step_index": step_index,
                        "stream": stream,
                        "line": line
                    });
                    realtime_hub.broadcast(payload.to_string()).await;
                }
                Ok(AgentToControllerMessage::JobFinished { job_id, status, outputs, step_summaries, error_message }) => {
                    info!(job_id = %job_id, status = %status, "Agent finished job");
                    let outputs_json = serde_json::to_value(outputs).unwrap_or_else(|_| serde_json::json!({}));

                    let _ = sqlx::query(
                        r#"
                        UPDATE jobs
                        SET status = $1,
                            outputs = $2,
                            step_summaries = $3,
                            error_message = $4,
                            finished_at = NOW()
                        WHERE id = $5
                        "#
                    )
                    .bind(&status)
                    .bind(outputs_json)
                    .bind(&step_summaries)
                    .bind(error_message)
                    .bind(job_id)
                    .execute(&pool)
                    .await;

                    let pipeline_res: Result<Option<(Uuid,)>, _> = sqlx::query_as(
                        "SELECT pipeline_id FROM jobs WHERE id = $1"
                    )
                    .bind(job_id)
                    .fetch_optional(&pool)
                    .await;

                    if let Ok(Some((pipeline_id,))) = pipeline_res {
                        let remaining: Result<(i64,), _> = sqlx::query_as(
                            "SELECT COUNT(*) FROM jobs WHERE pipeline_id = $1 AND status IN ('pending_dependencies', 'pending_agent', 'running', 'assigned')"
                        )
                        .bind(pipeline_id)
                        .fetch_one(&pool)
                        .await;

                        if let Ok((rem_count,)) = remaining {
                            if rem_count == 0 {
                                let failed_count: Result<(i64,), _> = sqlx::query_as(
                                    "SELECT COUNT(*) FROM jobs WHERE pipeline_id = $1 AND status = 'failed'"
                                )
                                .bind(pipeline_id)
                                .fetch_one(&pool)
                                .await;

                                let final_pipe_status = if let Ok((f,)) = failed_count {
                                    if f > 0 { "failed" } else { "success" }
                                } else {
                                    "success"
                                };

                                let _ = sqlx::query(
                                    "UPDATE pipelines SET status = $1, finished_at = NOW() WHERE id = $2"
                                )
                                .bind(final_pipe_status)
                                .bind(pipeline_id)
                                .execute(&pool)
                                .await;
                            }
                        }
                    }
                }
                Err(err) => {
                    error!(error = %err, "Failed to parse agent message");
                }
            }
        }
    }

    send_task.abort();

    if let Some(agent_id) = authenticated_agent_id {
        let _ = agent_service.mark_agent_offline(agent_id).await;
    }
}
