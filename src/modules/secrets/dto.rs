use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SecretRecord {
    pub id: Uuid,
    pub scope_type: String, // 'global' | 'project' | 'service'
    pub project_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub key: String,
    pub encrypted_value: String,
    pub iv: String,
    pub auth_tag: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecryptedSecret {
    pub key: String,
    pub value: String,
    pub scope_type: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSecretRequest {
    pub scope_type: String, // 'global' | 'project' | 'service'
    pub project_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    #[validate(length(min = 1, max = 150, message = "Key must be 1-150 characters"))]
    pub key: String,
    #[validate(length(min = 1, message = "Value cannot be empty"))]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretSummaryResponse {
    pub id: Uuid,
    pub scope_type: String,
    pub project_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
