pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::{
    routing::{get, delete},
    Router,
};
use std::sync::Arc;

pub use repository::{PostgresSecretRepository, SecretRepository};
pub use service::SecretService;

pub fn router(service: Arc<SecretService>) -> Router {
    Router::new()
        .route("/secrets", get(handler::list_secrets_handler).post(handler::create_secret_handler))
        .route("/secrets/:id", delete(handler::delete_secret_handler))
        .with_state(service)
}
