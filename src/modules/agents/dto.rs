use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AgentRecord {
    pub id: Uuid,
    pub name: String,
    pub agent_token: String,
    pub tags: serde_json::Value, // e.g. ["linux", "docker"]
    pub status: String,          // 'online' | 'offline' | 'busy'
    pub socket_id: Option<String>,
    pub os_info: Option<serde_json::Value>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterAgentRequest {
    #[validate(length(min = 2, max = 100, message = "Agent name must be 2-100 characters"))]
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StepDefinition {
    pub name: String,
    pub run: Option<String>,
    pub container_build: Option<ContainerBuildConfig>,
    pub when: Option<String>, // 'on_success' | 'always' | 'on_failure'
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerBuildConfig {
    pub image: String,
    pub dockerfile: Option<String>,
    pub tags: Vec<String>,
    pub registry_auth: Option<RegistryAuth>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryAuth {
    pub server: String,
    pub username: String,
    pub password: String,
}

/// Messages sent from Agent to Controller over WebSocket
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AgentToControllerMessage {
    Register {
        token: String,
        name: String,
        tags: Vec<String>,
        os_info: serde_json::Value,
    },
    Heartbeat,
    JobAccepted {
        job_id: Uuid,
    },
    LogStream {
        job_id: Uuid,
        step_index: usize,
        stream: String, // 'stdout' | 'stderr' | 'system'
        line: String,
    },
    JobFinished {
        job_id: Uuid,
        status: String, // 'success' | 'failed' | 'cancelled'
        outputs: HashMap<String, String>,
        step_summaries: String,
        error_message: Option<String>,
    },
}

/// Messages sent from Controller to Agent over WebSocket
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ControllerToAgentMessage {
    Registered {
        agent_id: Uuid,
    },
    AssignJob {
        job_id: Uuid,
        pipeline_id: Uuid,
        repo_url: String,
        git_ref: String,
        git_commit_sha: String,
        timeout_seconds: u64,
        steps: Vec<StepDefinition>,
        secrets: HashMap<String, String>,
        env: HashMap<String, String>,
    },
    CancelJob {
        job_id: Uuid,
    },
    Error {
        message: String,
    },
}

