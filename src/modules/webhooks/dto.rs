use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenericWebhookPayload {
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub repository: Option<WebhookRepositoryInfo>,
    pub commits: Option<Vec<WebhookCommitInfo>>,
    pub head_commit: Option<WebhookCommitInfo>,
    pub action: Option<String>,
    pub release: Option<WebhookReleaseInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebhookRepositoryInfo {
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub clone_url: Option<String>,
    pub ssh_url: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebhookCommitInfo {
    pub id: String,
    pub message: Option<String>,
    pub added: Option<Vec<String>>,
    pub removed: Option<Vec<String>>,
    pub modified: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebhookReleaseInfo {
    pub tag_name: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExaVaultWebhookPayload {
    pub event: String, // e.g. "secrets.updated"
    #[serde(default)]
    pub project_slug: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub changed_keys: Vec<String>,
    #[serde(default)]
    pub deleted_keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExaVaultWebhookResult {
    pub synced_services: usize,
    pub synced_projects: usize,
    pub total_keys_updated: usize,
}


