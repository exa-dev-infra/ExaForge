pub mod agent;
pub mod config;
pub mod constants;
pub mod database;
pub mod modules;
pub mod pkg;

use axum::{extract::State, routing::get, Json, Router};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tower_http::cors::CorsLayer;

use crate::{
    config::AppConfig,
    modules::{
        agents::{
            repository::PostgresAgentRepository,
            router as agents_router,
            service::AgentService,
        },
        auth::{handler::auth_router, service::AuthService},
        pipelines::{
            repository::PostgresPipelineRepository,
            router as pipelines_router,
            service::PipelineService,
        },
        projects::{
            repository::PostgresProjectRepository,
            router as projects_router,
            service::ProjectService,
        },
        realtime::realtime_router,
        secrets::{
            repository::PostgresSecretRepository,
            router as secrets_router,
            service::SecretService,
        },
        user::{handler::user_router, service::UserService},
        web::{web_router, WebContext},
        webhooks::{router as webhooks_router, service::WebhookService},
    },
    pkg::{
        idempotency::{idempotency_layer, IdempotencyStore},
        logging::middleware::create_trace_layer,
        realtime::RealtimeHub,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub pool: PgPool,
    pub hub: Arc<RealtimeHub>,
}

pub async fn health_check_handler(State(state): State<AppState>) -> Json<Value> {
    let mut is_healthy = true;

    let db_status = match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => "connected",
        Err(_) => {
            is_healthy = false;
            "error"
        }
    };

    Json(json!({
        "status": if is_healthy { "healthy" } else { "degraded" },
        "app": state.config.app_name,
        "environment": state.config.environment,
        "database": db_status,
        "timestamp": Utc::now().to_rfc3339()
    }))
}

pub fn build_app(config: AppConfig, pool: PgPool, hub: Arc<RealtimeHub>) -> Router {
    let state = AppState {
        config: config.clone(),
        pool: pool.clone(),
        hub: hub.clone(),
    };

    // Repositories
    let project_repo = Arc::new(PostgresProjectRepository);
    let secret_repo = Arc::new(PostgresSecretRepository);
    let agent_repo = Arc::new(PostgresAgentRepository);
    let pipeline_repo = Arc::new(PostgresPipelineRepository);

    // Services
    let project_service = Arc::new(ProjectService::new(pool.clone(), project_repo));
    let secret_service = Arc::new(SecretService::new(pool.clone(), secret_repo, &config.master_key));
    let agent_service = Arc::new(AgentService::new(pool.clone(), agent_repo));
    let pipeline_service = Arc::new(PipelineService::new(
        pool.clone(),
        pipeline_repo,
        project_service.clone(),
        secret_service.clone(),
        agent_service.clone(),
    ));
    let webhook_service = Arc::new(WebhookService::new(project_service.clone(), pipeline_service.clone()));

    let user_service = Arc::new(UserService::with_default_repo(pool.clone()));
    let auth_service = Arc::new(AuthService::with_default_repos(pool.clone(), config.clone()));

    // REST API Routes
    let mut api_routes = Router::new()
        .nest("/auth", auth_router(auth_service))
        .nest("/users", user_router(user_service))
        .nest("/realtime", realtime_router(hub.clone()))
        .merge(projects_router(project_service.clone()))
        .merge(secrets_router(secret_service.clone()))
        .merge(agents_router(agent_service.clone(), hub.clone(), pool.clone()))
        .merge(pipelines_router(pipeline_service.clone(), project_service.clone()))
        .merge(webhooks_router(webhook_service));

    if config.idempotency_enabled {
        let idempotency_store = Arc::new(IdempotencyStore::new(
            Duration::from_secs(config.idempotency_lifetime_hours * 3600),
            Duration::from_secs(config.idempotency_lock_timeout_seconds),
        ));
        api_routes = api_routes.layer(axum::middleware::from_fn(move |req, next| {
            idempotency_layer(idempotency_store.clone(), req, next)
        }));
    }

    // Askama Web Context for SSR Dashboard
    let web_ctx = WebContext {
        project_service,
        pipeline_service,
        agent_service,
        secret_service,
    };

    let health_router = Router::new()
        .route("/health", get(health_check_handler))
        .with_state(state);

    Router::new()
        .merge(web_router(web_ctx))
        .merge(health_router)
        .nest("/api/v1", api_routes)
        .layer(create_trace_layer::<axum::body::Body>())
        .layer(CorsLayer::permissive())
}
