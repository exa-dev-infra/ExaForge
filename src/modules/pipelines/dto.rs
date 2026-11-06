use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct PipelineRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub service_id: Uuid,
    pub trigger_type: String,
    pub trigger_user_id: Option<Uuid>,
    pub git_ref: String,
    pub git_commit_sha: String,
    pub git_commit_message: Option<String>,
    pub concurrency_group: Option<String>,
    pub cancel_in_progress: bool,
    pub status: String,
    pub inputs: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct JobRecord {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub job_key: String,
    pub name: String,
    pub required_tags: serde_json::Value,
    pub needs: serde_json::Value,
    pub status: String,
    pub assigned_agent_id: Option<Uuid>,
    pub timeout_seconds: i32,
    pub inputs: serde_json::Value,
    pub outputs: serde_json::Value,
    pub step_summaries: Option<String>,
    pub error_message: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct JobLogRecord {
    pub id: i64,
    pub job_id: Uuid,
    pub step_index: i32,
    pub stream: String,
    pub line: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ManualTriggerRequest {
    pub git_ref: Option<String>,
    pub inputs: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineDetailResponse {
    pub pipeline: PipelineRecord,
    pub jobs: Vec<JobRecord>,
}

