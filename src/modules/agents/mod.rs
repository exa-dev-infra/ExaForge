pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::{
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::pkg::realtime::RealtimeHub;

pub use handler::AgentContext;
pub use repository::{AgentRepository, PostgresAgentRepository};
pub use service::AgentService;

pub fn router(service: Arc<AgentService>, realtime_hub: Arc<RealtimeHub>, pool: PgPool) -> Router {
    let ctx = AgentContext {
        service,
        realtime_hub,
        pool,
    };

    Router::new()
        .route("/api/v1/agents", get(handler::list_agents_handler).post(handler::register_agent_handler))
        .route("/api/v1/agents/ws", get(handler::agent_ws_handler))
        .with_state(ctx)
}
