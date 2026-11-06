use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get},
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    pkg::{
        auth::jwt::AdminUser,
        errors::AppError,
        response::BaseResponse,
        validation::ValidatedJson,
    },
};
use super::{
    dto::{AssignProjectMemberRequest, AssignServiceMemberRequest},
    service::RbacService,
};

pub async fn list_roles_handler(
    State(service): State<Arc<RbacService>>,
) -> Result<impl IntoResponse, AppError> {
    let roles = service.list_roles().await?;
    Ok(BaseResponse::success(roles, "System roles retrieved successfully"))
}

pub async fn list_project_members_handler(
    State(service): State<Arc<RbacService>>,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let members = service.list_project_members_detail(project_id).await?;
    Ok(BaseResponse::success(members, "Project members retrieved successfully"))
}

pub async fn assign_project_member_handler(
    _admin: AdminUser,
    State(service): State<Arc<RbacService>>,
    Path(project_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<AssignProjectMemberRequest>,
) -> Result<impl IntoResponse, AppError> {
    let member = service.assign_project_member(project_id, payload).await?;
    Ok(BaseResponse::created(member, "Project member assigned successfully"))
}

pub async fn remove_project_member_handler(
    _admin: AdminUser,
    State(service): State<Arc<RbacService>>,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    service.remove_project_member(project_id, user_id).await?;
    Ok(BaseResponse::success(true, "Project member removed successfully"))
}

pub async fn list_service_members_handler(
    State(service): State<Arc<RbacService>>,
    Path((_project_id, service_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let members = service.list_service_members_detail(service_id).await?;
    Ok(BaseResponse::success(members, "Service members retrieved successfully"))
}

pub async fn assign_service_member_handler(
    _admin: AdminUser,
    State(service): State<Arc<RbacService>>,
    Path((project_id, service_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(payload): ValidatedJson<AssignServiceMemberRequest>,
) -> Result<impl IntoResponse, AppError> {
    let member = service.assign_service_member(project_id, service_id, payload).await?;
    Ok(BaseResponse::created(member, "Service member assigned successfully"))
}

pub async fn remove_service_member_handler(
    _admin: AdminUser,
    State(service): State<Arc<RbacService>>,
    Path((_project_id, service_id, user_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    service.remove_service_member(service_id, user_id).await?;
    Ok(BaseResponse::success(true, "Service member removed successfully"))
}

pub fn rbac_router(service: Arc<RbacService>) -> Router {
    Router::new()
        .route("/rbac/roles", get(list_roles_handler))
        .route("/projects/:project_id/members", get(list_project_members_handler).post(assign_project_member_handler))
        .route("/projects/:project_id/members/:user_id", delete(remove_project_member_handler))
        .route("/projects/:project_id/services/:service_id/members", get(list_service_members_handler).post(assign_service_member_handler))
        .route("/projects/:project_id/services/:service_id/members/:user_id", delete(remove_service_member_handler))
        .with_state(service)
}
