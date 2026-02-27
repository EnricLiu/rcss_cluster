mod schema;
mod policy;
mod config;
mod image;
mod server;
pub mod composer;
pub mod team;

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "match_composer", about = "Match Composer HTTP server")]
struct Args {
    /// HTTP server listen address
    #[arg(long, env = "HOST", default_value = "0.0.0.0")]
    host: Ipv4Addr,

    /// HTTP server listen port
    #[arg(long, env = "PORT", default_value = "6657")]
    port: u16,

    /// Path to the hub/registry directory containing agent images
    #[arg(long, env = "HUB_PATH", default_value = "sidecars/match_composer/hub")]
    hub_path: PathBuf,

    /// Root directory for match logs
    #[arg(long, env = "LOG_ROOT", default_value = "logs")]
    log_root: PathBuf,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    log::info!("hub_path : {:?}", args.hub_path);
    log::info!("log_root : {:?}", args.log_root);

    let addr = SocketAddr::new(args.host.into(), args.port);

    server::listen(addr, args.hub_path, args.log_root).await;
}
