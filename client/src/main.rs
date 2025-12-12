mod agones;
mod room;
mod utils;
mod error;
mod proxy;
mod controller;

use std::sync::Arc;

use error::{Error, Result};
use crate::proxy::{ProxyServer, ProxyServerConfig};

pub const TCP_ADDR: &str = "0.0.0.0:6000";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let proxy_server = Arc::new(ProxyServer::new(ProxyServerConfig::default()));
    let listen_task = controller::listen(TCP_ADDR, proxy_server.clone()).await;
    tracing::info!("API server listening on http://{TCP_ADDR}");
    
    tokio::select! {
        _ = listen_task => {
            tracing::error!("API server stopped");
        },
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutting down");
            proxy_server.shutdown().await.unwrap();
        }
    }
}
