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
    dto::CreateSecretRequest,
    service::SecretService,
};

pub async fn list_secrets_handler(
    State(service): State<Arc<SecretService>>,
) -> Result<impl IntoResponse, AppError> {
    let summaries = service.list_secrets_summary().await?;
    Ok(BaseResponse::success(summaries, "Secrets retrieved successfully"))
}

pub async fn create_secret_handler(
    State(service): State<Arc<SecretService>>,
    ValidatedJson(payload): ValidatedJson<CreateSecretRequest>,
) -> Result<impl IntoResponse, AppError> {
    let secret = service.create_or_update_secret(payload).await?;
    Ok(BaseResponse::created(secret.id, "Secret saved successfully"))
}

pub async fn delete_secret_handler(
    State(service): State<Arc<SecretService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_secret(id).await?;
    Ok(BaseResponse::success(true, "Secret deleted successfully"))
}
