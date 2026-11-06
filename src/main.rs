use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

use exaforge::{
    agent::{AgentConfig, WorkerAgentDaemon},
    build_app,
    config::AppConfig,
    database::postgres::{create_postgres_pool, run_migrations},
    pkg::{logging::init_logger, realtime::RealtimeHub},
};

#[derive(Parser)]
#[command(
    name = "exaforge",
    version,
    about = "ExaForge: Lightweight Self-Hosted CI/CD Engine (Controller + Worker Agent)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Controller HTTP Server, Web Dashboard & WebSocket Hub (default)
    Serve,

    /// Start a Worker Agent daemon to execute assigned CI/CD jobs locally
    Agent {
        /// Controller WebSocket endpoint URL
        #[arg(
            short,
            long,
            env = "EXAFORGE_CONTROLLER_URL",
            default_value = "ws://localhost:8080/api/v1/agents/ws"
        )]
        controller_url: String,

        /// Worker Agent authentication token registered in Controller
        #[arg(short, long, env = "EXAFORGE_AGENT_TOKEN")]
        token: String,

        /// Human-readable agent display name
        #[arg(short, long, env = "EXAFORGE_AGENT_NAME", default_value = "local-agent")]
        name: String,

        /// Capability tags comma-separated (e.g. linux,docker,k8s,android,macos)
        #[arg(
            long,
            env = "EXAFORGE_AGENT_TAGS",
            default_value = "linux,docker"
        )]
        tags: String,

        /// Local workspace directory root for job execution
        #[arg(short, long, env = "EXAFORGE_WORKSPACE_DIR")]
        workspace_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = AppConfig::from_env();

    // Initialize structured JSON logger
    init_logger(&config, None);

    match cli.command.unwrap_or(Commands::Serve) {
        Commands::Serve => {
            info!(
                host = %config.host,
                port = %config.port,
                app = %config.app_name,
                "Starting ExaForge Controller Server (Web Dashboard + WebSocket Hub)..."
            );

            // Connect to PostgreSQL
            let pool = match create_postgres_pool(&config.database_url).await {
                Ok(pool) => {
                    info!("PostgreSQL database pool established");
                    if let Err(e) = run_migrations(&pool).await {
                        tracing::warn!(error = %e, "Migration check failed, continuing with existing schema");
                    } else {
                        info!("Database migrations executed successfully");
                    }
                    if let Err(e) = exaforge::seed_default_super_admin(&pool).await {
                        tracing::warn!(error = %e, "Default super admin seed failed");
                    }
                    pool
                }
                Err(err) => {
                    tracing::error!(
                        error = %err,
                        database_url = %config.database_url,
                        "Failed to connect to PostgreSQL. Please ensure database is running."
                    );
                    return Err(Box::new(err) as Box<dyn std::error::Error>);
                }
            };

            // Initialize Realtime Hub
            let hub = Arc::new(RealtimeHub::default());
            let bridge = exaforge::pkg::realtime::create_realtime_bridge(&config, hub.sender()).await;
            hub.set_bridge(bridge).await;

            // Build Axum Router with Askama Web + REST APIs + Agent WS
            let app = build_app(config.clone(), pool, hub);

            let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
            let listener = TcpListener::bind(addr).await?;
            info!("Controller Web Dashboard & API is live on http://{}", addr);

            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

            info!("ExaForge Controller shutdown completed cleanly");
        }
        Commands::Agent {
            controller_url,
            token,
            name,
            tags,
            workspace_dir,
        } => {
            let tags_list: Vec<String> = tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let workspace_root = workspace_dir.unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".exaforge").join("workspaces")
            });

            let agent_cfg = AgentConfig {
                controller_ws_url: controller_url,
                token,
                name,
                tags: tags_list,
                workspace_root,
            };

            let daemon = WorkerAgentDaemon::new(agent_cfg);
            daemon.run().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        }
    }

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
            tracing::info!("Received Ctrl+C, gracefully terminating Controller...");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, gracefully terminating Controller...");
        },
    }
}
