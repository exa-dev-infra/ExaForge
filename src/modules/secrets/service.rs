use std::collections::HashMap;
use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    pkg::{
        crypto::{decrypt_secret, derive_master_key, encrypt_secret},
        errors::AppError,
    },
};
use super::{
    dto::{CreateSecretRequest, SecretRecord, SecretSummaryResponse},
    repository::SecretRepository,
};

#[derive(Clone)]
pub struct SecretService {
    pool: PgPool,
    repo: Arc<dyn SecretRepository>,
    master_key: [u8; 32],
}

impl SecretService {
    pub fn new(pool: PgPool, repo: Arc<dyn SecretRepository>, raw_master_key: &str) -> Self {
        let master_key = derive_master_key(raw_master_key);
        Self { pool, repo, master_key }
    }

    pub async fn create_or_update_secret(&self, req: CreateSecretRequest) -> Result<SecretRecord, AppError> {
        let (encrypted_value, iv, auth_tag) = encrypt_secret(&req.value, &self.master_key)?;

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

    /// Resolves decrypted secrets for a target project and service.
    /// Hierarchy override: Service secrets > Project secrets > Global secrets.
    /// Strictly excludes any other service's secrets!
    pub async fn resolve_secrets_for_job(
        &self,
        project_id: Uuid,
        service_id: Uuid,
    ) -> Result<HashMap<String, String>, AppError> {
        let records = self.repo.resolve_hierarchical_secrets(&self.pool, project_id, service_id).await?;

        let mut resolved: HashMap<String, String> = HashMap::new();

        // Since query returns global -> project -> service, later entries override earlier ones
        for record in records {
            let decrypted_val = decrypt_secret(
                &record.encrypted_value,
                &record.iv,
                &record.auth_tag,
                &self.master_key,
            )?;
            resolved.insert(record.key, decrypted_val);
        }

        Ok(resolved)
    }
}
