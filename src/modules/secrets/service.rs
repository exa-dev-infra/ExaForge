use std::collections::HashMap;
use std::sync::Arc;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::pkg::{
    crypto::{decrypt_secret, derive_master_key, encrypt_secret},
    errors::AppError,
    exavault::ExaVaultClient,
};
use super::{
    dto::{CreateSecretRequest, SecretRecord, SecretSummaryResponse, SyncExaVaultRequest, SyncExaVaultResponse},
    repository::SecretRepository,
};

#[derive(Clone)]
pub struct SecretService {
    pool: PgPool,
    repo: Arc<dyn SecretRepository>,
    master_key: [u8; 32],
    exavault_client: Arc<ExaVaultClient>,
}

impl SecretService {
    pub fn new(
        pool: PgPool,
        repo: Arc<dyn SecretRepository>,
        raw_master_key: &str,
        exavault_client: Arc<ExaVaultClient>,
    ) -> Self {
        let master_key = derive_master_key(raw_master_key);
        Self {
            pool,
            repo,
            master_key,
            exavault_client,
        }
    }

    pub async fn create_or_update_secret(&self, req: CreateSecretRequest) -> Result<SecretRecord, AppError> {
        let (encrypted_value, iv, auth_tag) = encrypt_secret(&req.value, &self.master_key)?;
        let source = req.source.as_deref().unwrap_or("manual");

        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let record = self.repo.upsert_secret(
            &mut tx,
            &req.scope_type,
            req.project_id,
            req.service_id,
            &req.key,
            &encrypted_value,
            &iv,
            &auth_tag,
            Some(source),
        ).await?;

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(record)
    }

    pub async fn list_secrets_summary(&self) -> Result<Vec<SecretSummaryResponse>, AppError> {
        let records = self.repo.list_all(&self.pool).await?;
        Ok(records.into_iter().map(|r| SecretSummaryResponse {
            id: r.id,
            scope_type: r.scope_type,
            project_id: r.project_id,
            service_id: r.service_id,
            key: r.key,
            source: r.source,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect())
    }

    pub async fn delete_secret(&self, id: Uuid) -> Result<bool, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let deleted = self.repo.delete_secret(&mut tx, id).await?;

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(deleted)
    }

    /// Synchronizes secrets directly from ExaVault into ExaForge.
    /// Encrypts each key-value pair with AES-256-GCM and persists with source = 'exavault'.
    /// Also updates the target project or service's exavault_url and exavault_token for auto-sync.
    pub async fn sync_from_exavault(&self, req: SyncExaVaultRequest) -> Result<SyncExaVaultResponse, AppError> {
        if req.scope_type == "service" && req.service_id.is_none() {
            return Err(AppError::bad_request("Service ID is required when scope_type is service", "INVALID_SCOPE"));
        }

        info!(scope = %req.scope_type, project_id = %req.project_id, "Syncing secrets from ExaVault...");
        let exported = self.exavault_client.export_secrets_json(
            req.exavault_url.as_deref(),
            &req.exavault_token,
        ).await?;

        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let mut keys = Vec::new();
        for (key, val) in &exported {
            let (encrypted_value, iv, auth_tag) = encrypt_secret(val, &self.master_key)?;
            self.repo.upsert_secret(
                &mut tx,
                &req.scope_type,
                Some(req.project_id),
                req.service_id,
                key,
                &encrypted_value,
                &iv,
                &auth_tag,
                Some("exavault"),
            ).await?;
            keys.push(key.clone());
        }

        // Persist exavault_url & exavault_token on project or service
        if req.scope_type == "service" {
            if let Some(sid) = req.service_id {
                sqlx::query(
                    "UPDATE services SET exavault_url = $1, exavault_token = $2, updated_at = NOW() WHERE id = $3"
                )
                .bind(&req.exavault_url)
                .bind(&req.exavault_token)
                .bind(sid)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::database(format!("Failed to update service ExaVault config: {}", e)))?;
            }
        } else if req.scope_type == "project" {
            sqlx::query(
                "UPDATE projects SET exavault_url = $1, exavault_token = $2, updated_at = NOW() WHERE id = $3"
            )
            .bind(&req.exavault_url)
            .bind(&req.exavault_token)
            .bind(req.project_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to update project ExaVault config: {}", e)))?;
        }

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        info!(count = keys.len(), "Successfully synchronized ExaVault secrets");
        Ok(SyncExaVaultResponse {
            synced_count: keys.len(),
            keys,
        })
    }

    /// Resolves decrypted secrets for a target project and service.
    /// Hierarchy override: Service Manual > Live Service ExaVault > Synced Service ExaVault >
    ///                     Project Manual > Live Project ExaVault > Synced Project ExaVault > Global Secrets.
    /// Strictly excludes any other service's secrets!
    pub async fn resolve_secrets_for_job(
        &self,
        project_id: Uuid,
        service_id: Uuid,
    ) -> Result<HashMap<String, String>, AppError> {
        let records = self.repo.resolve_hierarchical_secrets(&self.pool, project_id, service_id).await?;

        let mut resolved: HashMap<String, String> = HashMap::new();

        // 1. Seed with stored secrets from DB (Global -> Project -> Service, exavault then manual)
        for record in &records {
            let decrypted_val = decrypt_secret(
                &record.encrypted_value,
                &record.iv,
                &record.auth_tag,
                &self.master_key,
            )?;
            resolved.insert(record.key.clone(), decrypted_val);
        }

        // 2. Dynamic live auto-injection from ExaVault if project or service has exavault_token configured
        let project_row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT exavault_url, exavault_token FROM projects WHERE id = $1"
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        if let Some((url, Some(token))) = project_row {
            if !token.trim().is_empty() {
                match self.exavault_client.export_secrets_json(url.as_deref(), &token).await {
                    Ok(live_secrets) => {
                        info!(project_id = %project_id, count = live_secrets.len(), "Auto-injected live ExaVault project secrets into pipeline job");
                        for (k, v) in live_secrets {
                            resolved.insert(k, v);
                        }
                    }
                    Err(e) => {
                        warn!(project_id = %project_id, error = %e, "Could not fetch live ExaVault project secrets; using fallback cached secrets");
                    }
                }
            }
        }

        let service_row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT exavault_url, exavault_token FROM services WHERE id = $1"
        )
        .bind(service_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        if let Some((url, Some(token))) = service_row {
            if !token.trim().is_empty() {
                match self.exavault_client.export_secrets_json(url.as_deref(), &token).await {
                    Ok(live_secrets) => {
                        info!(service_id = %service_id, count = live_secrets.len(), "Auto-injected live ExaVault service secrets into pipeline job");
                        for (k, v) in live_secrets {
                            resolved.insert(k, v);
                        }
                    }
                    Err(e) => {
                        warn!(service_id = %service_id, error = %e, "Could not fetch live ExaVault service secrets; using fallback cached secrets");
                    }
                }
            }
        }

        // 3. Highest priority: Re-apply manual service-scoped secrets so specific service secrets always override
        for record in &records {
            if record.scope_type == "service" && record.source.as_deref() != Some("exavault") {
                if let Ok(decrypted_val) = decrypt_secret(
                    &record.encrypted_value,
                    &record.iv,
                    &record.auth_tag,
                    &self.master_key,
                ) {
                    resolved.insert(record.key.clone(), decrypted_val);
                }
            }
        }

        Ok(resolved)
    }
}
