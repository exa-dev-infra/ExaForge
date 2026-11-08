use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    modules::{
        pipelines::{dto::PipelineRecord, service::PipelineService},
        projects::service::ProjectService,
        secrets::{dto::SyncExaVaultRequest, service::SecretService},
    },
    pkg::{
        crypto::verify_hmac_sha256,
        errors::AppError,
        realtime::RealtimeHub,
    },
};
use super::dto::{ExaVaultWebhookPayload, ExaVaultWebhookQuery, ExaVaultWebhookResult, GenericWebhookPayload};

#[derive(Clone)]
pub struct WebhookService {
    project_service: Arc<ProjectService>,
    pipeline_service: Arc<PipelineService>,
    secret_service: Arc<SecretService>,
    hub: Arc<RealtimeHub>,
}

impl WebhookService {
    pub fn new(
        project_service: Arc<ProjectService>,
        pipeline_service: Arc<PipelineService>,
        secret_service: Arc<SecretService>,
        hub: Arc<RealtimeHub>,
    ) -> Self {
        Self {
            project_service,
            pipeline_service,
            secret_service,
            hub,
        }
    }

    /// Processes incoming Git webhook payload, verifies signature, evaluates monorepo/multi-repo filters,
    /// and triggers applicable pipeline runs.
    pub async fn process_webhook(
        &self,
        raw_body: &[u8],
        signature_header: Option<&str>,
        payload: GenericWebhookPayload,
    ) -> Result<Vec<PipelineRecord>, AppError> {
        let all_services = self.project_service.list_all_services().await?;
        let mut triggered_pipelines = Vec::new();

        // Collect all touched files across commits
        let mut modified_files = Vec::new();
        if let Some(commits) = &payload.commits {
            for c in commits {
                if let Some(added) = &c.added { modified_files.extend(added.clone()); }
                if let Some(modded) = &c.modified { modified_files.extend(modded.clone()); }
                if let Some(removed) = &c.removed { modified_files.extend(removed.clone()); }
            }
        }
        if let Some(head) = &payload.head_commit {
            if let Some(added) = &head.added { modified_files.extend(added.clone()); }
            if let Some(modded) = &head.modified { modified_files.extend(modded.clone()); }
        }

        let git_ref = payload.git_ref.as_deref().unwrap_or("refs/heads/main");
        let is_tag = git_ref.starts_with("refs/tags/");
        let tag_name = if is_tag {
            git_ref.strip_prefix("refs/tags/").unwrap_or(git_ref)
        } else {
            payload.release.as_ref().and_then(|r| r.tag_name.as_deref()).unwrap_or("")
        };

        let commit_sha = payload.after.as_deref()
            .or_else(|| payload.head_commit.as_ref().map(|c| c.id.as_str()))
            .unwrap_or("HEAD");

        let commit_msg = payload.head_commit.as_ref()
            .and_then(|c| c.message.as_deref())
            .unwrap_or("Git webhook event");

        let repo_url = payload.repository.as_ref().and_then(|r| r.clone_url.as_deref().or(r.html_url.as_deref())).unwrap_or("");

        for service in all_services {
            // Check repo URL matching (loose match or contains)
            let repo_matches = service.repo_url.is_empty()
                || repo_url.is_empty()
                || service.repo_url.contains(repo_url)
                || repo_url.contains(&service.repo_url);

            if !repo_matches {
                continue;
            }

            // Verify HMAC signature if header provided and secret set
            if let Some(sig) = signature_header {
                if !service.webhook_secret.is_empty() && !verify_hmac_sha256(raw_body, &service.webhook_secret, sig) {
                    warn!(service = %service.slug, "Webhook signature verification failed");
                    continue;
                }
            }

            // Monorepo vs Multi-repo filtering
            let should_trigger = if is_tag && !tag_name.is_empty() {
                // Check tag prefix (e.g. fe-v* matches fe-v1.0.0)
                if let Some(prefix) = &service.tag_prefix {
                    if let Ok(pattern) = glob::Pattern::new(prefix) {
                        pattern.matches(tag_name)
                    } else {
                        tag_name.starts_with(prefix)
                    }
                } else {
                    true
                }
            } else if service.repo_type == "monorepo" {
                // Check path filters (e.g. apps/fe/**)
                let filters: Vec<String> = service.path_filters.as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                if filters.is_empty() {
                    true
                } else {
                    // Check if any modified file matches any pattern
                    modified_files.iter().any(|file_path| {
                        filters.iter().any(|pattern_str| {
                            glob::Pattern::new(pattern_str)
                                .map(|p| p.matches(file_path))
                                .unwrap_or(false)
                        })
                    })
                }
            } else {
                // Multi-repo: triggers on push
                true
            };

            if should_trigger {
                info!(
                    service_id = %service.id,
                    service_slug = %service.slug,
                    git_ref = %git_ref,
                    "Webhook trigger criteria matched! Triggering pipeline..."
                );

                let trigger_type = if is_tag { "webhook_tag" } else { "webhook_push" };

                let pipeline = self.pipeline_service.trigger_pipeline(
                    service.project_id,
                    service.id,
                    trigger_type,
                    None,
                    git_ref,
                    commit_sha,
                    Some(commit_msg),
                    None,
                    None,
                ).await?;

                triggered_pipelines.push(pipeline);
            }
        }

        Ok(triggered_pipelines)
    }

    /// Processes an incoming webhook event from ExaVault (e.g. `secrets.updated`),
    /// automatically synchronizing secrets for matching projects and services into ExaForge.
    pub async fn process_exavault_webhook(
        &self,
        query: ExaVaultWebhookQuery,
        payload: ExaVaultWebhookPayload,
    ) -> Result<ExaVaultWebhookResult, AppError> {
        info!(
            event = %payload.event,
            project = ?payload.project_slug,
            query_project = ?query.project_slug,
            query_project_id = ?query.project_id,
            changed = ?payload.changed_keys,
            "Received ExaVault webhook event"
        );

        // Ping / Healthcheck event: acknowledge immediately
        if payload.event == "ping" || payload.event == "health" {
            info!("Acknowledged ExaVault ping healthcheck");
            return Ok(ExaVaultWebhookResult {
                synced_services: 0,
                synced_projects: 0,
                total_keys_updated: 0,
            });
        }

        let all_projects = self.project_service.list_projects().await?;
        let all_services = self.project_service.list_all_services().await?;

        let mut synced_projects = 0;
        let mut synced_services = 0;
        let mut total_keys = 0;

        let target_project_slug = query.project_slug.as_deref().or(payload.project_slug.as_deref());

        // 1. Sync matching projects that have exavault_token configured
        for p in &all_projects {
            if let Some(pid) = query.project_id {
                if p.id != pid {
                    continue;
                }
            }

            if let Some(token) = &p.exavault_token {
                if !token.trim().is_empty() {
                    if let Some(target_slug) = target_project_slug {
                        if p.slug != target_slug {
                            continue;
                        }
                    }

                    match self.secret_service.sync_from_exavault(SyncExaVaultRequest {
                        scope_type: "project".to_string(),
                        project_id: p.id,
                        service_id: None,
                        exavault_url: p.exavault_url.clone(),
                        exavault_token: token.clone(),
                    }).await {
                        Ok(resp) => {
                            info!(project = %p.slug, count = resp.synced_count, "Auto-synced project secrets via ExaVault webhook");
                            synced_projects += 1;
                            total_keys += resp.synced_count;
                        }
                        Err(e) => {
                            warn!(project = %p.slug, error = %e, "Failed to auto-sync project secrets via ExaVault webhook");
                        }
                    }
                }
            }
        }

        // 2. Sync matching services that have exavault_token configured
        for s in &all_services {
            if let Some(sid) = query.service_id {
                if s.id != sid {
                    continue;
                }
            }
            if let Some(pid) = query.project_id {
                if s.project_id != pid {
                    continue;
                }
            }

            if let Some(token) = &s.exavault_token {
                if !token.trim().is_empty() {
                    if let Some(target_slug) = target_project_slug {
                        if let Some(parent) = all_projects.iter().find(|p| p.id == s.project_id) {
                            if parent.slug != target_slug {
                                continue;
                            }
                        }
                    }

                    match self.secret_service.sync_from_exavault(SyncExaVaultRequest {
                        scope_type: "service".to_string(),
                        project_id: s.project_id,
                        service_id: Some(s.id),
                        exavault_url: s.exavault_url.clone(),
                        exavault_token: token.clone(),
                    }).await {
                        Ok(resp) => {
                            info!(service = %s.slug, count = resp.synced_count, "Auto-synced service secrets via ExaVault webhook");
                            synced_services += 1;
                            total_keys += resp.synced_count;
                        }
                        Err(e) => {
                            warn!(service = %s.slug, error = %e, "Failed to auto-sync service secrets via ExaVault webhook");
                        }
                    }
                }
            }
        }

        // 3. Broadcast real-time update to web dashboard
        let broadcast_msg = serde_json::json!({
            "event": "exavault_secrets_synced",
            "synced_projects": synced_projects,
            "synced_services": synced_services,
            "total_keys_updated": total_keys,
        });
        self.hub.broadcast(broadcast_msg.to_string()).await;

        Ok(ExaVaultWebhookResult {
            synced_services,
            synced_projects,
            total_keys_updated: total_keys,
        })
    }
}
