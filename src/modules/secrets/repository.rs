use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::pkg::errors::AppError;
use super::dto::SecretRecord;

#[async_trait]
pub trait SecretRepository: Send + Sync {
    async fn upsert_secret(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        scope_type: &str,
        project_id: Option<Uuid>,
        service_id: Option<Uuid>,
        key: &str,
        encrypted_value: &str,
        iv: &str,
        auth_tag: &str,
        source: Option<&str>,
    ) -> Result<SecretRecord, AppError>;

    async fn find_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<SecretRecord>, AppError>;
    async fn list_all(&self, pool: &PgPool) -> Result<Vec<SecretRecord>, AppError>;
    async fn list_by_scope(
        &self,
        pool: &PgPool,
        scope_type: &str,
        project_id: Option<Uuid>,
        service_id: Option<Uuid>,
    ) -> Result<Vec<SecretRecord>, AppError>;

    /// Resolves all hierarchical secrets applicable for a specific project & service.
    /// Order: Global secrets -> Project secrets -> Service secrets.
    async fn resolve_hierarchical_secrets(
        &self,
        pool: &PgPool,
        project_id: Uuid,
        service_id: Uuid,
    ) -> Result<Vec<SecretRecord>, AppError>;

    async fn delete_secret(&self, tx: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<bool, AppError>;
}

pub struct PostgresSecretRepository;

#[async_trait]
impl SecretRepository for PostgresSecretRepository {
    async fn upsert_secret(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        scope_type: &str,
        project_id: Option<Uuid>,
        service_id: Option<Uuid>,
        key: &str,
        encrypted_value: &str,
        iv: &str,
        auth_tag: &str,
        source: Option<&str>,
    ) -> Result<SecretRecord, AppError> {
        let dummy_uuid = Uuid::nil();
        let _pid_coalesced = project_id.unwrap_or(dummy_uuid);
        let _sid_coalesced = service_id.unwrap_or(dummy_uuid);
        let src = source.unwrap_or("manual");

        let record = sqlx::query_as::<_, SecretRecord>(
            r#"
            INSERT INTO secrets (scope_type, project_id, service_id, key, encrypted_value, iv, auth_tag, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (scope_type, COALESCE(project_id, '00000000-0000-0000-0000-000000000000'::uuid), COALESCE(service_id, '00000000-0000-0000-0000-000000000000'::uuid), key)
            DO UPDATE SET
                encrypted_value = EXCLUDED.encrypted_value,
                iv = EXCLUDED.iv,
                auth_tag = EXCLUDED.auth_tag,
                source = EXCLUDED.source,
                updated_at = NOW()
            RETURNING id, scope_type, project_id, service_id, key, encrypted_value, iv, auth_tag, source, created_at, updated_at
            "#
        )
        .bind(scope_type)
        .bind(project_id)
        .bind(service_id)
        .bind(key)
        .bind(encrypted_value)
        .bind(iv)
        .bind(auth_tag)
        .bind(src)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("Failed to upsert secret: {}", e)))?;

        Ok(record)
    }

    async fn find_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<SecretRecord>, AppError> {
        let record = sqlx::query_as::<_, SecretRecord>(
            "SELECT * FROM secrets WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find secret: {}", e)))?;

        Ok(record)
    }

    async fn list_all(&self, pool: &PgPool) -> Result<Vec<SecretRecord>, AppError> {
        let records = sqlx::query_as::<_, SecretRecord>(
            "SELECT * FROM secrets ORDER BY scope_type, key ASC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list secrets: {}", e)))?;

        Ok(records)
    }

    async fn list_by_scope(
        &self,
        pool: &PgPool,
        scope_type: &str,
        project_id: Option<Uuid>,
        service_id: Option<Uuid>,
    ) -> Result<Vec<SecretRecord>, AppError> {
        let records = sqlx::query_as::<_, SecretRecord>(
            r#"
            SELECT * FROM secrets
            WHERE scope_type = $1
              AND ($2::uuid IS NULL OR project_id = $2)
              AND ($3::uuid IS NULL OR service_id = $3)
            ORDER BY key ASC
            "#
        )
        .bind(scope_type)
        .bind(project_id)
        .bind(service_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list secrets by scope: {}", e)))?;

        Ok(records)
    }

    async fn resolve_hierarchical_secrets(
        &self,
        pool: &PgPool,
        project_id: Uuid,
        service_id: Uuid,
    ) -> Result<Vec<SecretRecord>, AppError> {
        // Fetches global secrets, project secrets, and service secrets in priority order.
        // Within the same scope, 'exavault' secrets are ordered before 'manual' secrets,
        // so that manual overrides take precedence when inserted into the resolution map.
        let records = sqlx::query_as::<_, SecretRecord>(
            r#"
            SELECT * FROM secrets
            WHERE (scope_type = 'global' AND project_id IS NULL AND service_id IS NULL)
               OR (scope_type = 'project' AND project_id = $1 AND service_id IS NULL)
               OR (scope_type = 'service' AND project_id = $1 AND service_id = $2)
            ORDER BY
                CASE scope_type
                    WHEN 'global' THEN 1
                    WHEN 'project' THEN 2
                    WHEN 'service' THEN 3
                    ELSE 4
                END ASC,
                CASE COALESCE(source, 'manual')
                    WHEN 'exavault' THEN 1
                    WHEN 'manual' THEN 2
                    ELSE 3
                END ASC,
                created_at ASC
            "#
        )
        .bind(project_id)
        .bind(service_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to resolve hierarchical secrets: {}", e)))?;

        Ok(records)
    }

    async fn delete_secret(&self, tx: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<bool, AppError> {
        let rows = sqlx::query("DELETE FROM secrets WHERE id = $1")
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete secret: {}", e)))?
            .rows_affected();

        Ok(rows > 0)
    }
}
