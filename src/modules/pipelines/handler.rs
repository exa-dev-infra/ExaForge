use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    modules::projects::service::ProjectService,
    pkg::{
        errors::AppError,
        response::{BaseResponse, PaginatedResponse},
        validation::ValidatedJson,
    },
};
use super::{
    dto::{ManualTriggerRequest, PipelineHistoryQuery},
    service::PipelineService,
};

pub async fn trigger_pipeline_handler(
    State((pipeline_service, project_service)): State<(Arc<PipelineService>, Arc<ProjectService>)>,
    Path((project_slug, service_slug)): Path<(String, String)>,
    ValidatedJson(payload): ValidatedJson<ManualTriggerRequest>,
) -> Result<impl IntoResponse, AppError> {
    let project = project_service.get_project_by_slug(&project_slug).await?;
    let service = project_service.get_service_by_slug(project.id, &service_slug).await?;

    let git_ref = payload.git_ref.unwrap_or(service.default_branch);

    let pipeline = pipeline_service.trigger_pipeline(
        project.id,
        service.id,
        "manual",
        None,
        &git_ref,
        "HEAD",
        Some("Manual workflow dispatch trigger"),
        payload.inputs,
        None,
    ).await?;

    Ok(BaseResponse::created(pipeline, "Pipeline triggered successfully"))
}

pub async fn get_pipeline_handler(
    State((pipeline_service, _)): State<(Arc<PipelineService>, Arc<ProjectService>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let detail = pipeline_service.get_pipeline_detail(id).await?;
    Ok(BaseResponse::success(detail, "Pipeline retrieved successfully"))
}

pub async fn cancel_pipeline_handler(
    State((pipeline_service, _)): State<(Arc<PipelineService>, Arc<ProjectService>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    pipeline_service.cancel_pipeline(id).await?;
    Ok(BaseResponse::success(true, "Pipeline cancelled successfully"))
}

pub async fn get_job_handler(
    State((pipeline_service, _)): State<(Arc<PipelineService>, Arc<ProjectService>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let job = pipeline_service.get_job_detail(id).await?;
    Ok(BaseResponse::success(job, "Job retrieved successfully"))
}

pub async fn get_job_logs_handler(
    State((pipeline_service, _)): State<(Arc<PipelineService>, Arc<ProjectService>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let logs = pipeline_service.get_job_logs(id).await?;
    Ok(BaseResponse::success(logs, "Job logs retrieved successfully"))
}

pub async fn list_pipelines_handler(
    State((pipeline_service, _)): State<(Arc<PipelineService>, Arc<ProjectService>)>,
    Query(query): Query<PipelineHistoryQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let page = (offset / limit) as u64 + 1;

    let items = pipeline_service.list_pipeline_history(
        query.project_id,
        query.service_id,
        query.status.as_deref().filter(|s| !s.is_empty() && *s != "all"),
        limit,
        offset,
    ).await?;

    let total = pipeline_service.count_pipeline_history(
        query.project_id,
        query.service_id,
        query.status.as_deref().filter(|s| !s.is_empty() && *s != "all"),
    ).await? as u64;

    Ok(PaginatedResponse::new(items, page, limit as u64, total, "Pipelines retrieved successfully"))
}

pub async fn list_service_pipelines_handler(
    State((pipeline_service, _)): State<(Arc<PipelineService>, Arc<ProjectService>)>,
    Path(service_id): Path<Uuid>,
    Query(query): Query<PipelineHistoryQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let page = (offset / limit) as u64 + 1;

    let items = pipeline_service.list_pipeline_history(
        query.project_id,
        Some(service_id),
        query.status.as_deref().filter(|s| !s.is_empty() && *s != "all"),
        limit,
        offset,
    ).await?;

    let total = pipeline_service.count_pipeline_history(
        query.project_id,
        Some(service_id),
        query.status.as_deref().filter(|s| !s.is_empty() && *s != "all"),
    ).await? as u64;

    Ok(PaginatedResponse::new(items, page, limit as u64, total, "Service pipelines retrieved successfully"))
}
