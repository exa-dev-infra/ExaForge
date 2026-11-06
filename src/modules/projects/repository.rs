use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::pkg::errors::AppError;
use super::dto::{ProjectRecord, ServiceRecord};

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create_project(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: &str,
        slug: &str,
        description: Option<&str>,
        exavault_url: Option<&str>,
        exavault_token: Option<&str>,
    ) -> Result<ProjectRecord, AppError>;

    async fn find_project_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<ProjectRecord>, AppError>;
    async fn find_project_by_slug(&self, pool: &PgPool, slug: &str) -> Result<Option<ProjectRecord>, AppError>;
    async fn list_projects(&self, pool: &PgPool) -> Result<Vec<ProjectRecord>, AppError>;
    async fn delete_project(&self, tx: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<bool, AppError>;

    async fn create_service(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: Uuid,
        name: &str,
        slug: &str,
        repo_url: &str,
        repo_type: &str,
        path_filters: serde_json::Value,
        tag_prefix: Option<&str>,
        default_branch: &str,
        webhook_secret: &str,
        pipeline_file: &str,
        exavault_url: Option<&str>,
        exavault_token: Option<&str>,
    ) -> Result<ServiceRecord, AppError>;

    async fn find_service_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<ServiceRecord>, AppError>;
    async fn find_service_by_slug(&self, pool: &PgPool, project_id: Uuid, slug: &str) -> Result<Option<ServiceRecord>, AppError>;
    async fn list_services_by_project(&self, pool: &PgPool, project_id: Uuid) -> Result<Vec<ServiceRecord>, AppError>;
    async fn list_all_services(&self, pool: &PgPool) -> Result<Vec<ServiceRecord>, AppError>;
    async fn delete_service(&self, tx: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<bool, AppError>;
}

pub struct PostgresProjectRepository;

#[async_trait]
impl ProjectRepository for PostgresProjectRepository {
    async fn create_project(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: &str,
        slug: &str,
        description: Option<&str>,
        exavault_url: Option<&str>,
        exavault_token: Option<&str>,
    ) -> Result<ProjectRecord, AppError> {
        let record = sqlx::query_as::<_, ProjectRecord>(
            r#"
            INSERT INTO projects (name, slug, description, exavault_url, exavault_token)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(name)
        .bind(slug)
        .bind(description)
        .bind(exavault_url)
        .bind(exavault_token)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("Failed to create project: {}", e)))?;

        Ok(record)
    }

    async fn find_project_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<ProjectRecord>, AppError> {
        let record = sqlx::query_as::<_, ProjectRecord>(
            "SELECT * FROM projects WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find project: {}", e)))?;

        Ok(record)
    }

    async fn find_project_by_slug(&self, pool: &PgPool, slug: &str) -> Result<Option<ProjectRecord>, AppError> {
        let record = sqlx::query_as::<_, ProjectRecord>(
            "SELECT * FROM projects WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find project by slug: {}", e)))?;

        Ok(record)
    }

    async fn list_projects(&self, pool: &PgPool) -> Result<Vec<ProjectRecord>, AppError> {
        let records = sqlx::query_as::<_, ProjectRecord>(
            "SELECT * FROM projects ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list projects: {}", e)))?;

        Ok(records)
    }

    async fn delete_project(&self, tx: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<bool, AppError> {
        let rows_affected = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete project: {}", e)))?
            .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn create_service(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: Uuid,
        name: &str,
        slug: &str,
        repo_url: &str,
        repo_type: &str,
        path_filters: serde_json::Value,
        tag_prefix: Option<&str>,
        default_branch: &str,
        webhook_secret: &str,
        pipeline_file: &str,
        exavault_url: Option<&str>,
        exavault_token: Option<&str>,
    ) -> Result<ServiceRecord, AppError> {
        let record = sqlx::query_as::<_, ServiceRecord>(
            r#"
            INSERT INTO services (
                project_id, name, slug, repo_url, repo_type, path_filters,
                tag_prefix, default_branch, webhook_secret, pipeline_file,
                exavault_url, exavault_token
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#
        )
        .bind(project_id)
        .bind(name)
        .bind(slug)
        .bind(repo_url)
        .bind(repo_type)
        .bind(path_filters)
        .bind(tag_prefix)
        .bind(default_branch)
        .bind(webhook_secret)
        .bind(pipeline_file)
        .bind(exavault_url)
        .bind(exavault_token)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("Failed to create service: {}", e)))?;

        Ok(record)
    }

    async fn find_service_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<ServiceRecord>, AppError> {
        let record = sqlx::query_as::<_, ServiceRecord>(
            "SELECT * FROM services WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find service: {}", e)))?;

        Ok(record)
    }

    async fn find_service_by_slug(&self, pool: &PgPool, project_id: Uuid, slug: &str) -> Result<Option<ServiceRecord>, AppError> {
        let record = sqlx::query_as::<_, ServiceRecord>(
            "SELECT * FROM services WHERE project_id = $1 AND slug = $2"
        )
        .bind(project_id)
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find service by slug: {}", e)))?;

        Ok(record)
    }

    async fn list_services_by_project(&self, pool: &PgPool, project_id: Uuid) -> Result<Vec<ServiceRecord>, AppError> {
        let records = sqlx::query_as::<_, ServiceRecord>(
            "SELECT * FROM services WHERE project_id = $1 ORDER BY created_at ASC"
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list services: {}", e)))?;

        Ok(records)
    }

    async fn list_all_services(&self, pool: &PgPool) -> Result<Vec<ServiceRecord>, AppError> {
        let records = sqlx::query_as::<_, ServiceRecord>(
            "SELECT * FROM services ORDER BY created_at ASC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list all services: {}", e)))?;

        Ok(records)
    }

    async fn delete_service(&self, tx: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<bool, AppError> {
        let rows_affected = sqlx::query("DELETE FROM services WHERE id = $1")
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete service: {}", e)))?
            .rows_affected();

        Ok(rows_affected > 0)
    }
}

