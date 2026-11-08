use axum::{
    body::Bytes,
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::pkg::{errors::AppError, response::BaseResponse};
use super::{
    dto::{ExaVaultWebhookPayload, ExaVaultWebhookQuery, GenericWebhookPayload},
    service::WebhookService,
};

pub async fn git_webhook_handler(
    State(service): State<Arc<WebhookService>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let signature_header = headers.get("X-Hub-Signature-256")
        .or_else(|| headers.get("x-hub-signature-256"))
        .and_then(|v| v.to_str().ok());

    let payload: GenericWebhookPayload = serde_json::from_slice(&body)
        .map_err(|e| AppError::bad_request(format!("Invalid webhook JSON payload: {}", e), "INVALID_PAYLOAD"))?;

    let triggered = service.process_webhook(&body, signature_header, payload).await?;

    Ok(BaseResponse::success(
        triggered,
        "Webhook processed successfully",
    ))
}

pub async fn exavault_webhook_handler(
    State(service): State<Arc<WebhookService>>,
    Query(query): Query<ExaVaultWebhookQuery>,
    _headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let payload: ExaVaultWebhookPayload = serde_json::from_slice(&body)
        .map_err(|e| AppError::bad_request(format!("Invalid ExaVault webhook JSON payload: {}", e), "INVALID_PAYLOAD"))?;

    let result = service.process_exavault_webhook(query, payload).await?;

    Ok(BaseResponse::success(
        result,
        "ExaVault webhook processed and secrets auto-synchronized",
    ))
}
