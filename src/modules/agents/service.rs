use std::{
    collections::HashMap,
    sync::Arc,
};
use sqlx::PgPool;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    constants::codes::CODE_AGENT_NOT_FOUND,
    pkg::{
        crypto::generate_agent_token,
        errors::AppError,
    },
};
use super::{
    dto::{AgentRecord, ControllerToAgentMessage, RegisterAgentRequest},
    repository::AgentRepository,
};

pub type AgentSender = mpsc::UnboundedSender<ControllerToAgentMessage>;

#[derive(Clone)]
pub struct AgentService {
    pool: PgPool,
    repo: Arc<dyn AgentRepository>,
    // In-memory active agent connections: Agent ID -> Channel Sender
    active_agents: Arc<RwLock<HashMap<Uuid, AgentSender>>>,
}

impl AgentService {
    pub fn new(pool: PgPool, repo: Arc<dyn AgentRepository>) -> Self {
        Self {
            pool,
            repo,
            active_agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_agent(&self, req: RegisterAgentRequest) -> Result<AgentRecord, AppError> {
        let token = generate_agent_token();
        let tags_json = serde_json::to_value(req.tags).unwrap_or_else(|_| serde_json::json!([]));

        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let agent = self.repo.create_agent(&mut tx, &req.name, &token, tags_json).await?;

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(agent)
    }

    pub async fn authenticate_by_token(&self, token: &str) -> Result<Option<AgentRecord>, AppError> {
        self.repo.find_by_token(&self.pool, token).await
    }

    pub async fn list_agents(&self) -> Result<Vec<AgentRecord>, AppError> {
        self.repo.list_agents(&self.pool).await
    }

    pub async fn get_agent(&self, id: Uuid) -> Result<AgentRecord, AppError> {
        self.repo.find_by_id(&self.pool, id).await?
            .ok_or_else(|| AppError::not_found("Agent not found", CODE_AGENT_NOT_FOUND))
    }

    pub async fn mark_agent_online(
        &self,
        agent_id: Uuid,
        socket_id: &str,
        os_info: serde_json::Value,
        sender: AgentSender,
    ) -> Result<(), AppError> {
        self.repo.update_status(&self.pool, agent_id, "online", Some(socket_id), Some(os_info)).await?;
        self.active_agents.write().await.insert(agent_id, sender);
        info!(agent_id = %agent_id, "Worker Agent connected and marked online");
        Ok(())
    }

    pub async fn mark_agent_offline(&self, agent_id: Uuid) -> Result<(), AppError> {
        self.repo.update_status(&self.pool, agent_id, "offline", None, None).await?;
        self.active_agents.write().await.remove(&agent_id);
        info!(agent_id = %agent_id, "Worker Agent disconnected and marked offline");
        Ok(())
    }

    pub async fn heartbeat(&self, agent_id: Uuid) -> Result<(), AppError> {
        self.repo.update_heartbeat(&self.pool, agent_id).await
    }

    pub async fn find_matching_online_agent(&self, required_tags: &[String]) -> Result<Option<AgentRecord>, AppError> {
        self.repo.find_matching_online_agent(&self.pool, required_tags).await
    }

    pub async fn send_to_agent(&self, agent_id: Uuid, msg: ControllerToAgentMessage) -> bool {
        let agents = self.active_agents.read().await;
        if let Some(sender) = agents.get(&agent_id) {
            sender.send(msg).is_ok()
        } else {
            warn!(agent_id = %agent_id, "Agent has no active connection sender");
            false
        }
    }
}

