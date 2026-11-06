use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::codes::CODE_PERMISSION_DENIED,
    modules::user::repository::UserRepository,
    pkg::errors::AppError,
};
use super::{
    dto::{AssignProjectMemberRequest, AssignServiceMemberRequest, ProjectMembershipRecord, ServiceMembershipRecord},
    repository::RbacRepository,
};

#[derive(Clone)]
pub struct RbacService {
    pool: PgPool,
    repo: Arc<dyn RbacRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl RbacService {
    pub fn new(pool: PgPool, repo: Arc<dyn RbacRepository>, user_repo: Arc<dyn UserRepository>) -> Self {
        Self { pool, repo, user_repo }
    }

    /// Checks whether user is a global Super Admin
    pub async fn is_super_admin(&self, user_id: Uuid) -> Result<bool, AppError> {
        let user = self.user_repo.find_by_id(&self.pool, user_id).await?;
        Ok(user.map(|u| u.role == "admin" || u.role == "super_admin").unwrap_or(false))
    }

    /// Verifies if user has permission to manage project or view project
    pub async fn check_project_access(
        &self,
        user_id: Uuid,
        project_id: Uuid,
        require_maintainer: bool,
    ) -> Result<bool, AppError> {
        if self.is_super_admin(user_id).await? {
            return Ok(true);
        }

        let membership = self.repo.get_project_membership(&self.pool, user_id, project_id).await?;
        match membership {
            Some(mem) => {
                if require_maintainer {
                    Ok(mem.role == "maintainer")
                } else {
                    Ok(true) // maintainer or viewer
                }
            }
            None => Ok(false),
        }
    }

    /// Verifies if user has permission for a specific service (e.g. coffee:fe)
    pub async fn check_service_access(
        &self,
        user_id: Uuid,
        project_id: Uuid,
        service_id: Uuid,
        is_write_or_trigger: bool,
        is_staging_only: bool,
    ) -> Result<bool, AppError> {
        // 1. Super Admin has unrestricted access
        if self.is_super_admin(user_id).await? {
            return Ok(true);
        }

        // 2. Project Maintainer has full access to all services in that project
        if let Some(pm) = self.repo.get_project_membership(&self.pool, user_id, project_id).await? {
            if pm.role == "maintainer" {
                return Ok(true);
            }
            if !is_write_or_trigger && pm.role == "viewer" {
                return Ok(true);
            }
        }

        // 3. Service Developer (Scoped Role: coffee:fe)
        if let Some(sm) = self.repo.get_service_membership(&self.pool, user_id, service_id).await? {
            if sm.role == "developer" {
                return Ok(true);
            }
            if sm.role == "viewer" {
                if !is_write_or_trigger {
                    return Ok(true);
                }
                if is_staging_only && sm.can_deploy_staging_only {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Enforces service access or throws Forbidden AppError
    pub async fn require_service_access(
        &self,
        user_id: Uuid,
        project_id: Uuid,
        service_id: Uuid,
        is_write_or_trigger: bool,
        is_staging_only: bool,
    ) -> Result<(), AppError> {
        if !self.check_service_access(user_id, project_id, service_id, is_write_or_trigger, is_staging_only).await? {
            return Err(AppError::forbidden(
                "Access denied: you do not have permission for this project/service",
                CODE_PERMISSION_DENIED,
            ));
        }
        Ok(())
    }

    pub async fn assign_project_member(
        &self,
        project_id: Uuid,
        req: AssignProjectMemberRequest,
    ) -> Result<ProjectMembershipRecord, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let member = self.repo.upsert_project_membership(
            &mut tx,
            req.user_id,
            project_id,
            &req.role,
        ).await?;

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(member)
    }

    pub async fn assign_service_member(
        &self,
        project_id: Uuid,
        service_id: Uuid,
        req: AssignServiceMemberRequest,
    ) -> Result<ServiceMembershipRecord, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let member = self.repo.upsert_service_membership(
            &mut tx,
            req.user_id,
            project_id,
            service_id,
            &req.role,
            req.can_deploy_staging_only.unwrap_or(false),
        ).await?;

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(member)
    }

    pub async fn list_project_members(&self, project_id: Uuid) -> Result<Vec<ProjectMembershipRecord>, AppError> {
        self.repo.list_project_members(&self.pool, project_id).await
    }

    pub async fn list_service_members(&self, service_id: Uuid) -> Result<Vec<ServiceMembershipRecord>, AppError> {
        self.repo.list_service_members(&self.pool, service_id).await
    }
}
