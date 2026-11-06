use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::pkg::errors::AppError;
use super::dto::{ProjectMembershipRecord, ServiceMembershipRecord};

#[async_trait]
pub trait RbacRepository: Send + Sync {
    async fn upsert_project_membership(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        project_id: Uuid,
        role: &str,
    ) -> Result<ProjectMembershipRecord, AppError>;

    async fn get_project_membership(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<Option<ProjectMembershipRecord>, AppError>;

    async fn list_project_members(
        &self,
        pool: &PgPool,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMembershipRecord>, AppError>;

    async fn delete_project_membership(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<bool, AppError>;

    async fn upsert_service_membership(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        project_id: Uuid,
        service_id: Uuid,
        role: &str,
        can_deploy_staging_only: bool,
    ) -> Result<ServiceMembershipRecord, AppError>;

    async fn get_service_membership(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        service_id: Uuid,
    ) -> Result<Option<ServiceMembershipRecord>, AppError>;

    async fn list_service_members(
        &self,
        pool: &PgPool,
        service_id: Uuid,
    ) -> Result<Vec<ServiceMembershipRecord>, AppError>;

    async fn delete_service_membership(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        service_id: Uuid,
    ) -> Result<bool, AppError>;
}

pub struct PostgresRbacRepository;

#[async_trait]
impl RbacRepository for PostgresRbacRepository {
    async fn upsert_project_membership(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        project_id: Uuid,
        role: &str,
    ) -> Result<ProjectMembershipRecord, AppError> {
        let record = sqlx::query_as::<_, ProjectMembershipRecord>(
            r#"
            INSERT INTO project_memberships (user_id, project_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (project_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            RETURNING id, user_id, project_id, role, created_at
            "#
        )
        .bind(user_id)
        .bind(project_id)
        .bind(role)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("Failed to upsert project membership: {}", e)))?;

        Ok(record)
    }

    async fn get_project_membership(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<Option<ProjectMembershipRecord>, AppError> {
        let record = sqlx::query_as::<_, ProjectMembershipRecord>(
            "SELECT id, user_id, project_id, role, created_at FROM project_memberships WHERE user_id = $1 AND project_id = $2"
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to get project membership: {}", e)))?;

        Ok(record)
    }

    async fn list_project_members(
        &self,
        pool: &PgPool,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMembershipRecord>, AppError> {
        let records = sqlx::query_as::<_, ProjectMembershipRecord>(
            "SELECT id, user_id, project_id, role, created_at FROM project_memberships WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list project members: {}", e)))?;

        Ok(records)
    }

    async fn delete_project_membership(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<bool, AppError> {
        let rows = sqlx::query("DELETE FROM project_memberships WHERE user_id = $1 AND project_id = $2")
            .bind(user_id)
            .bind(project_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete project membership: {}", e)))?
            .rows_affected();

        Ok(rows > 0)
    }

    async fn upsert_service_membership(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        project_id: Uuid,
        service_id: Uuid,
        role: &str,
        can_deploy_staging_only: bool,
    ) -> Result<ServiceMembershipRecord, AppError> {
        let record = sqlx::query_as::<_, ServiceMembershipRecord>(
            r#"
            INSERT INTO service_memberships (user_id, project_id, service_id, role, can_deploy_staging_only)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (service_id, user_id)
            DO UPDATE SET role = EXCLUDED.role, can_deploy_staging_only = EXCLUDED.can_deploy_staging_only
            RETURNING id, user_id, project_id, service_id, role, can_deploy_staging_only, created_at
            "#
        )
        .bind(user_id)
        .bind(project_id)
        .bind(service_id)
        .bind(role)
        .bind(can_deploy_staging_only)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("Failed to upsert service membership: {}", e)))?;

        Ok(record)
    }

    async fn get_service_membership(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        service_id: Uuid,
    ) -> Result<Option<ServiceMembershipRecord>, AppError> {
        let record = sqlx::query_as::<_, ServiceMembershipRecord>(
            "SELECT id, user_id, project_id, service_id, role, can_deploy_staging_only, created_at FROM service_memberships WHERE user_id = $1 AND service_id = $2"
        )
        .bind(user_id)
        .bind(service_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to get service membership: {}", e)))?;

        Ok(record)
    }

    async fn list_service_members(
        &self,
        pool: &PgPool,
        service_id: Uuid,
    ) -> Result<Vec<ServiceMembershipRecord>, AppError> {
        let records = sqlx::query_as::<_, ServiceMembershipRecord>(
            "SELECT id, user_id, project_id, service_id, role, can_deploy_staging_only, created_at FROM service_memberships WHERE service_id = $1"
        )
        .bind(service_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list service members: {}", e)))?;

        Ok(records)
    }

    async fn delete_service_membership(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        service_id: Uuid,
    ) -> Result<bool, AppError> {
        let rows = sqlx::query("DELETE FROM service_memberships WHERE user_id = $1 AND service_id = $2")
            .bind(user_id)
            .bind(service_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete service membership: {}", e)))?
            .rows_affected();

        Ok(rows > 0)
    }
}
