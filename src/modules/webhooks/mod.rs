pub mod dto;
pub mod handler;
pub mod service;

use axum::{routing::post, Router};
use std::sync::Arc;

pub use service::WebhookService;

pub fn router(service: Arc<WebhookService>) -> Router {
    Router::new()
        .route("/webhooks/git", post(handler::git_webhook_handler))
        .with_state(service)
}

