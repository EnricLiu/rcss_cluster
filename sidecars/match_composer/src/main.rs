mod schema;
mod policy;
mod config;
mod image;
mod server;
pub mod composer;
pub mod team;

mod agones;
mod player;

use std::env;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use clap::Parser;
use crate::agones::Annotations;

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


    #[arg(short='f', long, env = "CONFIG_FILE", help = "Path to the ConfigV1 JSON file, exclusive with -a or --agones")]
    file: Option<PathBuf>,

    #[arg(short='a', long, env = "AGONES_ENABLED", default_value = "false", help = "Enable Agones SDK integration for GameServer lifecycle management")]
    agones: bool,

    #[arg(long, env = "AGONES_GRPC_PORT", default_value = "9357", help = "Port for Agones SDK gRPC communication")]
    agones_grpc_port: Option<u16>,

    #[arg(long, env = "AGONES_KEEP_ALIVE", default_value = "30", help = "Interval in seconds for sending keep-alive messages to Agones")]
    agones_keep_alive: Option<u64>
}

#[tokio::main]
async fn main() {
    unsafe { env::set_var("RUST_LOG", "debug") }
    env_logger::init();
    let args = Args::parse();

    if args.agones ^ args.file.is_none() {
        log::error!("Exact one of --agones or --file should be specified");
        std::process::exit(1);
    }

    let config = if args.agones {
        let mut agones_sdk = agones::Sdk::new(args.agones_grpc_port, args.agones_keep_alive.map(|s| Duration::from_secs(s)))
            .await.expect("Failed to initialize Agones SDK");

        let gs = agones_sdk.get_gameserver().await.unwrap();

        let meta = gs.object_meta.unwrap();
        let annotations = Annotations::from_map(meta.annotations);
        annotations.try_into().unwrap()
    } else {
        serde_json::from_str::<schema::v1::ConfigV1>(
            &std::fs::read_to_string(args.file.unwrap())
                .expect("Failed to read config file")
        ).expect("Failed to parse ConfigV1 from config file")
    };

    // let config = MatchComposerConfig::try_from(meta)
    //     .expect("Failed to parse MatchComposerConfig from GameServer metadata");

    log::info!("hub_path : {:?}", args.hub_path);
    log::info!("log_root : {:?}", args.log_root);

    let addr = SocketAddr::new(args.host.into(), args.port);

    server::listen(addr, config, args.hub_path, args.log_root).await;
}
