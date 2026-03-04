mod auth;
mod config;
mod controller;
mod k8s;

use clap::Parser;
use std::sync::Arc;

use config::Config;
use controller::allocate::AppState;
use k8s::K8sClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    // Parse configuration
    let config = Config::parse();
    config.validate()?;

    log::info!("Starting Allocator service");
    log::info!("Fleet: {}", config.fleet_name);
    log::info!("Namespace: {}", config.namespace);
    log::info!("Scheduling: {}", config.scheduling.as_str());
    log::info!("Bind address: {}", config.bind_address);
    log::info!(
        "Bot count range: {} - {}",
        config.min_bot_count,
        config.max_bot_count
    );

    // Initialize Kubernetes client
    log::info!("Initializing Kubernetes client");
    let k8s_client = K8sClient::new().await?;

    // Create app state
    let state = AppState {
        config: Arc::new(config.clone()),
        k8s_client,
    };

    // Create router
    let app = controller::create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    log::info!("Listening on {}", config.bind_address);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    log::info!("Server shutdown complete");
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log::info!("Shutdown signal received");
}
