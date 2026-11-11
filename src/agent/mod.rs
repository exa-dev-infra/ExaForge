pub mod git_cache;
pub mod masker;
pub mod process_group;
pub mod step_runner;

use futures_util::{SinkExt, StreamExt};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, watch, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

use crate::{
    modules::agents::dto::{AgentToControllerMessage, ControllerToAgentMessage},
    pkg::errors::AppError,
};
use self::{
    git_cache::GitCacheManager,
    masker::SecretMasker,
    step_runner::StepRunner,
};

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub controller_ws_url: String,
    pub token: String,
    pub name: String,
    pub tags: Vec<String>,
    pub workspace_root: PathBuf,
}

pub struct WorkerAgentDaemon {
    config: AgentConfig,
    active_cancel_tx: Arc<Mutex<Option<watch::Sender<bool>>>>,
}

impl WorkerAgentDaemon {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            active_cancel_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Runs the daemon loop with auto-reconnect
    pub async fn run(&self) -> Result<(), AppError> {
        info!(
            controller = %self.config.controller_ws_url,
            agent_name = %self.config.name,
            tags = ?self.config.tags,
            "Starting ExaForge Worker Agent daemon..."
        );

        loop {
            match self.connect_and_serve().await {
                Ok(_) => {
                    info!("Agent connection closed cleanly, reconnecting in 3 seconds...");
                }
                Err(err) => {
                    warn!(error = %err, "Agent connection error, reconnecting in 5 seconds...");
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_and_serve(&self) -> Result<(), AppError> {
        let (ws_stream, _) = connect_async(&self.config.controller_ws_url)
            .await
            .map_err(|e| AppError::internal(format!("Failed to connect to Controller WS: {}", e)))?;

        info!("Connected to Controller WebSocket successfully!");

        let (mut write_half, mut read_half) = ws_stream.split();
        let (to_controller_tx, mut to_controller_rx) = mpsc::unbounded_channel::<AgentToControllerMessage>();

        // 1. Send Register Message
        let os_info = serde_json::json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "family": std::env::consts::FAMILY,
        });

        let register_msg = AgentToControllerMessage::Register {
            token: self.config.token.clone(),
            name: self.config.name.clone(),
            tags: self.config.tags.clone(),
            os_info,
        };

        let reg_str = serde_json::to_string(&register_msg).unwrap();
        write_half.send(Message::Text(reg_str)).await
            .map_err(|e| AppError::internal(format!("Failed to send register: {}", e)))?;

        // 2. Writer task (forwards outgoing messages to Controller)
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = to_controller_rx.recv().await {
                if let Ok(json_str) = serde_json::to_string(&msg) {
                    if write_half.send(Message::Text(json_str)).await.is_err() {
                        break;
                    }
                }
            }
        });

        // 3. Heartbeat task (sends Heartbeat every 20s)
        let hb_tx = to_controller_tx.clone();
        let heartbeat_task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(20)).await;
                if hb_tx.send(AgentToControllerMessage::Heartbeat).is_err() {
                    break;
                }
            }
        });

        let git_cache_manager = Arc::new(GitCacheManager::new(GitCacheManager::default_dir()));

        // 4. Read loop
        while let Some(msg_result) = read_half.next().await {
            let msg = msg_result.map_err(|e| AppError::internal(format!("WS read error: {}", e)))?;

            if let Message::Text(text) = msg {
                let parsed: Result<ControllerToAgentMessage, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(ControllerToAgentMessage::Registered { agent_id }) => {
                        info!(agent_id = %agent_id, "Agent successfully registered and acknowledged by Controller");
                    }
                    Ok(ControllerToAgentMessage::AssignJob {
                        job_id,
                        pipeline_id: _,
                        repo_url,
                        git_ref,
                        git_commit_sha,
                        timeout_seconds,
                        steps,
                        secrets,
                        env,
                    }) => {
                        info!(job_id = %job_id, "Received job assignment from Controller");

                        let _ = to_controller_tx.send(AgentToControllerMessage::JobAccepted { job_id });

                        let (cancel_tx, cancel_rx) = watch::channel(false);
                        *self.active_cancel_tx.lock().await = Some(cancel_tx);

                        let agent_tx = to_controller_tx.clone();
                        let workspace_dir = self.config.workspace_root.join(job_id.to_string());
                        let git_cache = git_cache_manager.clone();

                        tokio::spawn(async move {
                            let masker = Arc::new(SecretMasker::new(&secrets));
                            let mut current_env = env;
                            for (k, v) in secrets {
                                current_env.insert(k, v);
                            }

                            let mut job_outputs = HashMap::new();
                            let mut overall_summary = String::new();
                            let mut job_failed = false;
                            let mut error_msg: Option<String> = None;

                            // A. Checkout workspace with incremental Git cache
                            let log_tx_git = agent_tx.clone();
                            let masker_git = masker.clone();
                            let log_line = move |line: String, is_err: bool| {
                                let masked = masker_git.mask(&line);
                                let stream = if is_err { "stderr" } else { "stdout" };
                                let _ = log_tx_git.send(AgentToControllerMessage::LogStream {
                                    job_id,
                                    step_index: 0,
                                    stream: stream.to_string(),
                                    line: masked,
                                });
                            };

                            log_line(format!("==> Synchronizing Git repository cache for {}", repo_url), false);
                            if let Err(e) = git_cache.checkout_workspace(&repo_url, &git_ref, &git_commit_sha, &workspace_dir).await {
                                log_line(format!("[error] Git checkout failed: {}", e), true);
                                job_failed = true;
                                error_msg = Some(e.to_string());
                            } else {
                                log_line("==> Workspace initialized successfully".to_string(), false);
                            }

                            // B. Run pipeline steps sequentially
                            if !job_failed {
                                for (idx, step) in steps.iter().enumerate() {
                                    let log_tx_step = agent_tx.clone();
                                    let masker_step = masker.clone();
                                    let cb = move |line: String, is_err: bool| {
                                        let masked = masker_step.mask(&line);
                                        let stream = if is_err { "stderr" } else { "stdout" };
                                        let _ = log_tx_step.send(AgentToControllerMessage::LogStream {
                                            job_id,
                                            step_index: idx + 1,
                                            stream: stream.to_string(),
                                            line: masked,
                                        });
                                    };

                                    let step_res = StepRunner::execute_step(
                                        step,
                                        &workspace_dir,
                                        &mut current_env,
                                        &mut job_outputs,
                                        &mut overall_summary,
                                        job_failed,
                                        Duration::from_secs(timeout_seconds),
                                        cancel_rx.clone(),
                                        cb,
                                    ).await;

                                    match step_res {
                                        Ok(true) => {}
                                        Ok(false) => {
                                            job_failed = true;
                                        }
                                        Err(e) => {
                                            job_failed = true;
                                            error_msg = Some(e.to_string());
                                        }
                                    }
                                }
                            }

                            // C. Clean up workspace
                            let _ = tokio::fs::remove_dir_all(&workspace_dir).await;

                            let final_status = if job_failed { "failed" } else { "success" };
                            info!(job_id = %job_id, status = final_status, "Job execution completed on worker agent");

                            let _ = agent_tx.send(AgentToControllerMessage::JobFinished {
                                job_id,
                                status: final_status.to_string(),
                                outputs: job_outputs,
                                step_summaries: overall_summary,
                                error_message: error_msg,
                            });
                        });
                    }
                    Ok(ControllerToAgentMessage::CancelJob { job_id }) => {
                        info!(job_id = %job_id, "Received CancelJob instruction, terminating active process group...");
                        let guard = self.active_cancel_tx.lock().await;
                        if let Some(tx) = &*guard {
                            let _ = tx.send(true);
                        }
                    }
                    Ok(ControllerToAgentMessage::Error { message }) => {
                        error!(message = %message, "Controller reported error");
                    }
                    Err(e) => {
                        warn!(error = %e, raw = %text, "Unknown message format from Controller");
                    }
                }
            }
        }

        heartbeat_task.abort();
        writer_task.abort();
        Ok(())
    }
}
