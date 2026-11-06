use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ProjectRecord {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ServiceRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub slug: String,
    pub repo_url: String,
    pub repo_type: String, // 'multi_repo' | 'monorepo'
    pub path_filters: serde_json::Value,
    pub tag_prefix: Option<String>,
    pub default_branch: String,
    pub webhook_secret: String,
    pub pipeline_file: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(min = 2, max = 100, message = "Project name must be 2-100 characters"))]
    pub name: String,
    #[validate(length(min = 2, max = 100, message = "Slug must be 2-100 characters"))]
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateServiceRequest {
    #[validate(length(min = 1, max = 100, message = "Service name must be 1-100 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 100, message = "Slug must be 1-100 characters"))]
    pub slug: String,
    #[validate(url(message = "repo_url must be a valid URL or git path"))]
    pub repo_url: String,
    pub repo_type: Option<String>,
    pub path_filters: Option<Vec<String>>,
    pub tag_prefix: Option<String>,
    pub default_branch: Option<String>,
    pub pipeline_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDetailResponse {
    pub project: ProjectRecord,
    pub services: Vec<ServiceRecord>,
}

