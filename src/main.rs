use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

use exaforge::{
    build_app,
    config::AppConfig,
    database::postgres::{create_postgres_pool, run_migrations},
    pkg::{logging::init_logger, realtime::RealtimeHub},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();

    // 1. Initialize OpenTelemetry Tracer if enabled
    let (otel_provider, otel_err) = match exaforge::pkg::telemetry::init_telemetry(&config) {
        Ok(provider) => (provider, None),
        Err(err) => (None, Some(err)),
    };

    // 2. Initialize structured JSON logging (Mandated by monorepo rules)
    init_logger(&config, otel_provider.as_ref());

    if let Some(err) = otel_err {
        tracing::warn!(error = %err, "Failed to initialize OpenTelemetry tracer provider");
    }

    info!(
        host = %config.host,
        port = %config.port,
        otel_enabled = config.otel_enabled,
        "Starting Rust Axum server..."
    );

    // 2. Connect to PostgreSQL
    let pool = match create_postgres_pool(&config.database_url).await {
        Ok(pool) => {
            info!("Database connection pool established successfully");
            // Run automatic migrations
            if let Err(e) = run_migrations(&pool).await {
                tracing::warn!(error = %e, "Migration check failed, continuing with existing schema");
            } else {
                info!("Database migrations executed successfully");
            }
            pool
        }
        Err(err) => {
            tracing::error!(
                error = %err,
                database_url = %config.database_url,
                "Failed to connect to database. Please ensure PostgreSQL is running."
            );
            return Err(Box::new(err) as Box<dyn std::error::Error>);
        }
    };

    // 3. Initialize Realtime Hub with Distributed Bridge (Redis / RabbitMQ / Local)
    let hub = Arc::new(RealtimeHub::default());
    let bridge = exaforge::pkg::realtime::create_realtime_bridge(&config, hub.sender()).await;
    hub.set_bridge(bridge).await;

    // 4. Build Axum Router
    let app = build_app(config.clone(), pool, hub);

    // 5. Start listener with graceful shutdown
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Server is listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    info!("Server shutdown completed cleanly");
    exaforge::pkg::telemetry::shutdown_telemetry();
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, shutting down...");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down...");
        },
    }
}
