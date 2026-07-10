#[cfg(not(feature = "agones"))]
compile_error!("Allocator binary requires the 'agones' feature to be enabled. Please enable it in your Cargo.toml or build command.");

#[cfg(feature = "agones")]
mod k8s;
mod args;
mod utils;
mod schema;
mod metadata;
mod controller;
mod declaration;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use arcstr::ArcStr;
use clap::Parser;

use k8s::{GsSweepConfig, K8sClient};
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
    log::info!("    GameServer template path: {}",args.gs_template.display());
    log::info!("    Default mode: {:?}", args.default_mode);

    log::info!("Loading fleet template");
    k8s::init_fleet_template(&args.fleet_template)
        .map_err(|e| format!("Fleet template initialization failed: {e}"))?;

    log::info!("Loading GameServer template");
    k8s::init_gs_template(&args.gs_template)
        .map_err(|e| format!("GameServer template initialization failed: {e}"))?;

    log::info!("Initializing Kubernetes client");
    let namespace = ArcStr::from(&args.namespace);
    let k8s = K8sClient::new(
        namespace,
        args.k8s_n_retry,
        Duration::from_millis(args.k8s_retry_interval_ms),
    ).await?;

    let sweep_config = GsSweepConfig {
        interval: Duration::from_secs(args.gs_sweep_interval_s),
        ready_idle_ttl: duration_opt(args.gs_ready_idle_ttl_s),
        lease_ttl: duration_opt(args.gs_lease_ttl_s),
        hard_ttl: duration_opt(args.gs_hard_ttl_s),
    };
    if sweep_config.enabled() {
        tokio::spawn(k8s.clone().run_managed_gs_sweeper(sweep_config));
    }

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

fn duration_opt(seconds: u64) -> Option<Duration> {
    if seconds == 0 {
        None
    } else {
        Some(Duration::from_secs(seconds))
    }
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
