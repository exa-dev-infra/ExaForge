use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::codes::{CODE_CONFLICT, CODE_PROJECT_NOT_FOUND, CODE_SERVICE_NOT_FOUND},
    pkg::errors::AppError,
};
use super::{
    dto::{CreateProjectRequest, CreateServiceRequest, ProjectDetailResponse, ProjectRecord, ServiceRecord},
    repository::ProjectRepository,
};

#[derive(Clone)]
pub struct ProjectService {
    pool: PgPool,
    repo: Arc<dyn ProjectRepository>,
}

impl ProjectService {
    pub fn new(pool: PgPool, repo: Arc<dyn ProjectRepository>) -> Self {
        Self { pool, repo }
    }

    pub async fn create_project(&self, req: CreateProjectRequest) -> Result<ProjectRecord, AppError> {
        // Check slug uniqueness
        if self.repo.find_project_by_slug(&self.pool, &req.slug).await?.is_some() {
            return Err(AppError::conflict(
                format!("Project with slug '{}' already exists", req.slug),
                CODE_CONFLICT,
            ));
        }

        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let project = self.repo.create_project(
            &mut tx,
            &req.name,
            &req.slug,
            req.description.as_deref(),
        ).await?;

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(project)
    }

    pub async fn get_project(&self, id: Uuid) -> Result<ProjectRecord, AppError> {
        self.repo.find_project_by_id(&self.pool, id).await?
            .ok_or_else(|| AppError::not_found("Project not found", CODE_PROJECT_NOT_FOUND))
    }

    pub async fn get_project_by_slug(&self, slug: &str) -> Result<ProjectRecord, AppError> {
        self.repo.find_project_by_slug(&self.pool, slug).await?
            .ok_or_else(|| AppError::not_found("Project not found", CODE_PROJECT_NOT_FOUND))
    }

    pub async fn get_project_detail(&self, slug: &str) -> Result<ProjectDetailResponse, AppError> {
        let project = self.get_project_by_slug(slug).await?;
        let services = self.repo.list_services_by_project(&self.pool, project.id).await?;
        Ok(ProjectDetailResponse { project, services })
    }

    pub async fn list_projects(&self) -> Result<Vec<ProjectRecord>, AppError> {
        self.repo.list_projects(&self.pool).await
    }

    pub async fn delete_project(&self, id: Uuid) -> Result<bool, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let deleted = self.repo.delete_project(&mut tx, id).await?;

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(deleted)
    }

    pub async fn create_service(
        &self,
        project_id: Uuid,
        req: CreateServiceRequest,
    ) -> Result<ServiceRecord, AppError> {
        // Validate project exists
        let _ = self.get_project(project_id).await?;

        // Check service slug uniqueness in project
        if self.repo.find_service_by_slug(&self.pool, project_id, &req.slug).await?.is_some() {
            return Err(AppError::conflict(
                format!("Service with slug '{}' already exists in this project", req.slug),
                CODE_CONFLICT,
            ));
        }

        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        // Generate webhook secret (random 32 hex chars)
        let mut rand_bytes = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut rand_bytes);
        let webhook_secret = hex::encode(rand_bytes);

        let path_filters_json = serde_json::to_value(req.path_filters.unwrap_or_default())
            .unwrap_or_else(|_| serde_json::json!([]));

        let service = self.repo.create_service(
            &mut tx,
            project_id,
            &req.name,
            &req.slug,
            &req.repo_url,
            req.repo_type.as_deref().unwrap_or("multi_repo"),
            path_filters_json,
            req.tag_prefix.as_deref(),
            req.default_branch.as_deref().unwrap_or("main"),
            &webhook_secret,
            req.pipeline_file.as_deref().unwrap_or(".ci.yaml"),
        ).await?;

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(service)
    }

    pub async fn get_service(&self, id: Uuid) -> Result<ServiceRecord, AppError> {
        self.repo.find_service_by_id(&self.pool, id).await?
            .ok_or_else(|| AppError::not_found("Service not found", CODE_SERVICE_NOT_FOUND))
    }

    pub async fn get_service_by_slug(&self, project_id: Uuid, slug: &str) -> Result<ServiceRecord, AppError> {
        self.repo.find_service_by_slug(&self.pool, project_id, slug).await?
            .ok_or_else(|| AppError::not_found("Service not found", CODE_SERVICE_NOT_FOUND))
    }

    pub async fn list_services(&self, project_id: Uuid) -> Result<Vec<ServiceRecord>, AppError> {
        self.repo.list_services_by_project(&self.pool, project_id).await
    }

    pub async fn list_all_services(&self) -> Result<Vec<ServiceRecord>, AppError> {
        self.repo.list_all_services(&self.pool).await
    }

    pub async fn delete_service(&self, id: Uuid) -> Result<bool, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let deleted = self.repo.delete_service(&mut tx, id).await?;

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(deleted)
    }
}

