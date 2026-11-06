use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum SystemRole {
    SuperAdmin,
    User,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ProjectRole {
    Maintainer,
    Viewer,
}

impl ToString for ProjectRole {
    fn to_string(&self) -> String {
        match self {
            ProjectRole::Maintainer => "maintainer".to_string(),
            ProjectRole::Viewer => "viewer".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ServiceRole {
    Developer,
    Viewer,
}

impl ToString for ServiceRole {
    fn to_string(&self) -> String {
        match self {
            ServiceRole::Developer => "developer".to_string(),
            ServiceRole::Viewer => "viewer".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ProjectMembershipRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ServiceMembershipRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub service_id: Uuid,
    pub role: String,
    pub can_deploy_staging_only: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AssignProjectMemberRequest {
    pub user_id: Uuid,
    pub role: String, // 'maintainer' | 'viewer'
}

#[derive(Debug, Deserialize, Validate)]
pub struct AssignServiceMemberRequest {
    pub user_id: Uuid,
    pub role: String, // 'developer' | 'viewer'
    pub can_deploy_staging_only: Option<bool>,
}
