use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::pkg::{
    errors::AppError,
    response::BaseResponse,
    validation::ValidatedJson,
};
use super::{
    dto::{CreateProjectRequest, CreateServiceRequest},
    service::ProjectService,
};

pub async fn list_projects_handler(
    State(service): State<Arc<ProjectService>>,
) -> Result<impl IntoResponse, AppError> {
    let projects = service.list_projects().await?;
    Ok(BaseResponse::success(projects, "Projects retrieved successfully"))
}

pub async fn create_project_handler(
    State(service): State<Arc<ProjectService>>,
    ValidatedJson(payload): ValidatedJson<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    let project = service.create_project(payload).await?;
    Ok(BaseResponse::created(project, "Project created successfully"))
}

pub async fn get_project_handler(
    State(service): State<Arc<ProjectService>>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let detail = service.get_project_detail(&slug).await?;
    Ok(BaseResponse::success(detail, "Project details retrieved"))
}

pub async fn delete_project_handler(
    State(service): State<Arc<ProjectService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_project(id).await?;
    Ok(BaseResponse::success(true, "Project deleted successfully"))
}

pub async fn list_services_handler(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let services = service.list_services(project_id).await?;
    Ok(BaseResponse::success(services, "Services retrieved successfully"))
}

pub async fn create_service_handler(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<CreateServiceRequest>,
) -> Result<impl IntoResponse, AppError> {
    let svc = service.create_service(project_id, payload).await?;
    Ok(BaseResponse::created(svc, "Service created successfully"))
}

pub async fn delete_service_handler(
    State(service): State<Arc<ProjectService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_service(id).await?;
    Ok(BaseResponse::success(true, "Service deleted successfully"))
}
