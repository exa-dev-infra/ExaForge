pub mod dto;
pub mod handler;
pub mod parser;
pub mod repository;
pub mod service;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::modules::projects::service::ProjectService;

pub use repository::{PipelineRepository, PostgresPipelineRepository};
pub use service::PipelineService;

pub fn router(service: Arc<PipelineService>, project_service: Arc<ProjectService>) -> Router {
    Router::new()
        .route(
            "/api/v1/projects/:project_slug/services/:service_slug/triggers",
            post(handler::trigger_pipeline_handler),
        )
        .route("/api/v1/pipelines/:id", get(handler::get_pipeline_handler))
        .route("/api/v1/pipelines/:id/cancel", post(handler::cancel_pipeline_handler))
        .route("/api/v1/jobs/:id", get(handler::get_job_handler))
        .route("/api/v1/jobs/:id/logs", get(handler::get_job_logs_handler))
        .with_state((service, project_service))
}

