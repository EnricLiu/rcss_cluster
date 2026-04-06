mod k8s;
mod auth;
mod args;
mod schema;
mod metadata;
mod controller;
mod declaration;

use std::net::SocketAddr;
use std::sync::Arc;
use arcstr::ArcStr;
use clap::Parser;

use k8s::K8sClient;
use args::Args;

use metadata::MetaData;
use controller::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let addr = SocketAddr::from((args.host, args.http_port));

    log::info!("Starting Allocator service");
    log::info!("    Bind address: {}", addr);
    log::info!("    Namespace: {}", args.namespace);
    log::info!("    Fleet template path: {}", args.fleet_template.display());

    log::info!("Loading fleet template");
    if let Err(e) = k8s::init_fleet_template(&args.fleet_template) {
        log::error!("Failed to load fleet template: {}", e);

        if let Ok(entries) = std::fs::read_dir("/mnt") {
            log::error!("Contents of /mnt:");
            for entry in entries.flatten() {
                log::error!("  - {}", entry.path().display());
            }
        } else {
            log::error!("Failed to read /mnt directory");
        }

        Err(e)?;
    }

    log::info!("Initializing Kubernetes client");
    let namespace = ArcStr::from(&args.namespace);
    let k8s = K8sClient::new(namespace).await?;

    let state = AppState {
        config: Arc::new(args.clone()),
        k8s,
    };

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    log::info!("Listening on {}", addr);

    let app = controller::route("/", state);
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
        use tokio::signal::unix;
        unix::signal(unix::SignalKind::terminate())
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
