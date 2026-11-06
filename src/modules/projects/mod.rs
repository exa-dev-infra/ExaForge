pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::{
    routing::{get, delete},
    Router,
};
use std::sync::Arc;

pub use service::ProjectService;
pub use repository::{ProjectRepository, PostgresProjectRepository};

pub fn router(service: Arc<ProjectService>) -> Router {
    Router::new()
        .route("/projects", get(handler::list_projects_handler).post(handler::create_project_handler))
        .route("/projects/:slug", get(handler::get_project_handler).delete(handler::delete_project_handler))
        .route("/projects/:project_id/services", get(handler::list_services_handler).post(handler::create_service_handler))
        .route("/services/:id", delete(handler::delete_service_handler))
        .with_state(service)
}
