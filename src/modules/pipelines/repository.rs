use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::pkg::errors::AppError;
use super::dto::{JobLogRecord, JobRecord, PipelineHistoryItem, PipelineRecord};

#[async_trait]
pub trait PipelineRepository: Send + Sync {
    async fn create_pipeline(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: Uuid,
        service_id: Uuid,
        trigger_type: &str,
        trigger_user_id: Option<Uuid>,
        git_ref: &str,
        git_commit_sha: &str,
        git_commit_message: Option<&str>,
        concurrency_group: Option<&str>,
        cancel_in_progress: bool,
        inputs: serde_json::Value,
    ) -> Result<PipelineRecord, AppError>;

    async fn find_pipeline_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<PipelineRecord>, AppError>;
    async fn list_pipelines_by_service(&self, pool: &PgPool, service_id: Uuid) -> Result<Vec<PipelineRecord>, AppError>;
    async fn list_recent_pipelines(&self, pool: &PgPool, limit: i64) -> Result<Vec<PipelineRecord>, AppError>;
    async fn list_pipeline_history(
        &self,
        pool: &PgPool,
        project_id: Option<Uuid>,
        service_id: Option<Uuid>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PipelineHistoryItem>, AppError>;
    async fn count_pipeline_history(
        &self,
        pool: &PgPool,
        project_id: Option<Uuid>,
        service_id: Option<Uuid>,
        status: Option<&str>,
    ) -> Result<i64, AppError>;

    async fn find_running_in_concurrency_group(
        &self,
        pool: &PgPool,
        project_id: Uuid,
        concurrency_group: &str,
    ) -> Result<Vec<PipelineRecord>, AppError>;

    async fn update_pipeline_status(
        &self,
        pool: &PgPool,
        id: Uuid,
        status: &str,
    ) -> Result<(), AppError>;

    async fn create_job(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        pipeline_id: Uuid,
        job_key: &str,
        name: &str,
        required_tags: serde_json::Value,
        needs: serde_json::Value,
        timeout_seconds: i32,
        inputs: serde_json::Value,
    ) -> Result<JobRecord, AppError>;

    async fn find_job_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<JobRecord>, AppError>;
    async fn list_jobs_by_pipeline(&self, pool: &PgPool, pipeline_id: Uuid) -> Result<Vec<JobRecord>, AppError>;
    async fn list_pending_agent_jobs(&self, pool: &PgPool) -> Result<Vec<JobRecord>, AppError>;

    async fn update_job_status(
        &self,
        pool: &PgPool,
        id: Uuid,
        status: &str,
        assigned_agent_id: Option<Uuid>,
    ) -> Result<(), AppError>;

    async fn get_job_logs(&self, pool: &PgPool, job_id: Uuid) -> Result<Vec<JobLogRecord>, AppError>;
}

pub struct PostgresPipelineRepository;

#[async_trait]
impl PipelineRepository for PostgresPipelineRepository {
    async fn create_pipeline(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: Uuid,
        service_id: Uuid,
        trigger_type: &str,
        trigger_user_id: Option<Uuid>,
        git_ref: &str,
        git_commit_sha: &str,
        git_commit_message: Option<&str>,
        concurrency_group: Option<&str>,
        cancel_in_progress: bool,
        inputs: serde_json::Value,
    ) -> Result<PipelineRecord, AppError> {
        let record = sqlx::query_as::<_, PipelineRecord>(
            r#"
            INSERT INTO pipelines (
                project_id, service_id, trigger_type, trigger_user_id,
                git_ref, git_commit_sha, git_commit_message, concurrency_group,
                cancel_in_progress, status, inputs, started_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'running', $10, NOW())
            RETURNING id, project_id, service_id, trigger_type, trigger_user_id,
                      git_ref, git_commit_sha, git_commit_message, concurrency_group,
                      cancel_in_progress, status, inputs, started_at, finished_at,
                      created_at, updated_at
            "#
        )
        .bind(project_id)
        .bind(service_id)
        .bind(trigger_type)
        .bind(trigger_user_id)
        .bind(git_ref)
        .bind(git_commit_sha)
        .bind(git_commit_message)
        .bind(concurrency_group)
        .bind(cancel_in_progress)
        .bind(inputs)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("Failed to create pipeline: {}", e)))?;

        Ok(record)
    }

    async fn find_pipeline_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<PipelineRecord>, AppError> {
        let record = sqlx::query_as::<_, PipelineRecord>(
            "SELECT * FROM pipelines WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find pipeline: {}", e)))?;

        Ok(record)
    }

    async fn list_pipelines_by_service(&self, pool: &PgPool, service_id: Uuid) -> Result<Vec<PipelineRecord>, AppError> {
        let records = sqlx::query_as::<_, PipelineRecord>(
            "SELECT * FROM pipelines WHERE service_id = $1 ORDER BY created_at DESC"
        )
        .bind(service_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list pipelines: {}", e)))?;

        Ok(records)
    }

    async fn list_recent_pipelines(&self, pool: &PgPool, limit: i64) -> Result<Vec<PipelineRecord>, AppError> {
        let records = sqlx::query_as::<_, PipelineRecord>(
            "SELECT * FROM pipelines ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list recent pipelines: {}", e)))?;

        Ok(records)
    }

    async fn list_pipeline_history(
        &self,
        pool: &PgPool,
        project_id: Option<Uuid>,
        service_id: Option<Uuid>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PipelineHistoryItem>, AppError> {
        let rows = sqlx::query_as::<_, PipelineHistoryItem>(
            r#"
            SELECT 
                p.id,
                p.project_id,
                pr.name as project_name,
                pr.slug as project_slug,
                p.service_id,
                s.name as service_name,
                s.slug as service_slug,
                p.trigger_type,
                p.git_ref,
                p.git_commit_sha,
                p.git_commit_message,
                p.status,
                p.started_at,
                p.finished_at,
                p.created_at
            FROM pipelines p
            JOIN projects pr ON pr.id = p.project_id
            JOIN services s ON s.id = p.service_id
            WHERE ($1::uuid IS NULL OR p.project_id = $1)
              AND ($2::uuid IS NULL OR p.service_id = $2)
              AND ($3::text IS NULL OR p.status = $3)
            ORDER BY p.created_at DESC
            LIMIT $4 OFFSET $5
            "#
        )
        .bind(project_id)
        .bind(service_id)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list pipeline history: {}", e)))?;

        Ok(rows)
    }

    async fn count_pipeline_history(
        &self,
        pool: &PgPool,
        project_id: Option<Uuid>,
        service_id: Option<Uuid>,
        status: Option<&str>,
    ) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM pipelines p
            WHERE ($1::uuid IS NULL OR p.project_id = $1)
              AND ($2::uuid IS NULL OR p.service_id = $2)
              AND ($3::text IS NULL OR p.status = $3)
            "#
        )
        .bind(project_id)
        .bind(service_id)
        .bind(status)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to count pipeline history: {}", e)))?;

        Ok(count.0)
    }

    async fn find_running_in_concurrency_group(
        &self,
        pool: &PgPool,
        project_id: Uuid,
        concurrency_group: &str,
    ) -> Result<Vec<PipelineRecord>, AppError> {
        let records = sqlx::query_as::<_, PipelineRecord>(
            "SELECT * FROM pipelines WHERE project_id = $1 AND concurrency_group = $2 AND status = 'running'"
        )
        .bind(project_id)
        .bind(concurrency_group)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find running concurrency group pipelines: {}", e)))?;

        Ok(records)
    }

    async fn update_pipeline_status(&self, pool: &PgPool, id: Uuid, status: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE pipelines SET status = $1, finished_at = NOW(), updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to update pipeline status: {}", e)))?;

        Ok(())
    }

    async fn create_job(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        pipeline_id: Uuid,
        job_key: &str,
        name: &str,
        required_tags: serde_json::Value,
        needs: serde_json::Value,
        timeout_seconds: i32,
        inputs: serde_json::Value,
    ) -> Result<JobRecord, AppError> {
        let record = sqlx::query_as::<_, JobRecord>(
            r#"
            INSERT INTO jobs (
                pipeline_id, job_key, name, required_tags, needs,
                status, timeout_seconds, inputs
            )
            VALUES ($1, $2, $3, $4, $5, 'pending_agent', $6, $7)
            RETURNING id, pipeline_id, job_key, name, required_tags, needs,
                      status, assigned_agent_id, timeout_seconds, inputs, outputs,
                      step_summaries, error_message, queued_at, started_at, finished_at
            "#
        )
        .bind(pipeline_id)
        .bind(job_key)
        .bind(name)
        .bind(required_tags)
        .bind(needs)
        .bind(timeout_seconds)
        .bind(inputs)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("Failed to create job: {}", e)))?;

        Ok(record)
    }

    async fn find_job_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<JobRecord>, AppError> {
        let record = sqlx::query_as::<_, JobRecord>(
            "SELECT * FROM jobs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find job: {}", e)))?;

        Ok(record)
    }

    async fn list_jobs_by_pipeline(&self, pool: &PgPool, pipeline_id: Uuid) -> Result<Vec<JobRecord>, AppError> {
        let records = sqlx::query_as::<_, JobRecord>(
            "SELECT * FROM jobs WHERE pipeline_id = $1 ORDER BY queued_at ASC"
        )
        .bind(pipeline_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list jobs: {}", e)))?;

        Ok(records)
    }

    async fn list_pending_agent_jobs(&self, pool: &PgPool) -> Result<Vec<JobRecord>, AppError> {
        let records = sqlx::query_as::<_, JobRecord>(
            "SELECT * FROM jobs WHERE status = 'pending_agent' ORDER BY queued_at ASC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list pending agent jobs: {}", e)))?;

        Ok(records)
    }

    async fn update_job_status(
        &self,
        pool: &PgPool,
        id: Uuid,
        status: &str,
        assigned_agent_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE jobs SET status = $1, assigned_agent_id = COALESCE($2, assigned_agent_id) WHERE id = $3"
        )
        .bind(status)
        .bind(assigned_agent_id)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update job status: {}", e)))?;

        Ok(())
    }

    async fn get_job_logs(&self, pool: &PgPool, job_id: Uuid) -> Result<Vec<JobLogRecord>, AppError> {
        let records = sqlx::query_as::<_, JobLogRecord>(
            "SELECT * FROM job_logs WHERE job_id = $1 ORDER BY id ASC"
        )
        .bind(job_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to get job logs: {}", e)))?;

        Ok(records)
    }
}

