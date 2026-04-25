mod args;
mod model;
mod config;
mod player;
mod policy;
mod server;
mod metadata;

pub mod info;
pub mod team;
pub mod composer;

use std::env;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use clap::Parser;
use log::{error, warn};
use allocator::declaration;
use common::utils::logging::{LoggingArgs, init_dual_logger, init_stdout_logger};

use crate::metadata::MetaData;
use crate::config::{MatchComposerConfig, RcssServerConfig};

fn init_logging(
    level: &'static str,
    log_args: &LoggingArgs,
    stdio_suffix: Option<PathBuf>
) -> std::io::Result<Option<PathBuf>> {
    let mut ret = None;

    match (log_args.try_resolve_log_root(), stdio_suffix) {
        (Ok(log_root), Some(stdio_suffix)) => {
            let log_file = log_root.join(stdio_suffix);
            if let Err(e) = init_dual_logger(&log_file, level) {
                eprintln!("[FATAL] Failed to initialize logger at {}: {}", log_file.display(), e);
                return Err(e);
            }
            ret = Some(log_root);
        }
        (Ok(_), None) => {
            eprintln!("[FATAL] Stdio log path is required when log root is specified");
            std::process::exit(1);
        }
        (Err(e), Some(stdio_suffix)) => {
            eprintln!("[Logging] Log root not specified, use relative path for stdio log: {}, Error: {e}", stdio_suffix.display());
            if let Err(e) = init_dual_logger(&stdio_suffix, level) {
                eprintln!("[FATAL] Failed to initialize logger at {}: {}", stdio_suffix.display(), e);
                return Err(e);
            }
        }
        _ => init_stdout_logger(level),
    };

    Ok(ret)
}

#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    let log_root = init_logging("info", &args.log_args, args.stdio_log_path).unwrap()
        .unwrap_or(env::current_dir().unwrap());

    if args.agones ^ args.file.is_none() {
        log::error!("Exact one of --agones or --file should be specified");
        std::process::exit(1);
    }

    let meta = if args.agones {
        let mut agones_sdk = agones::Sdk::new(args.agones_grpc_port, args.agones_keep_alive.map(|s| Duration::from_secs(s)))
            .await.expect("Failed to initialize Agones SDK");

        let gs = agones_sdk.get_gameserver().await.unwrap();

        let meta = gs.object_meta.unwrap();
        MetaData::try_from(meta).unwrap()
    } else {
        let config_v1 = serde_json::from_str::<allocator::schema::v1::ConfigV1>(
            &std::fs::read_to_string(args.file.unwrap())
                .expect("Failed to read config file")
        ).expect("Failed to parse ConfigV1 from config file");
        
        MetaData::from(
            allocator::MetaData::try_from(config_v1)
                .expect("Failed to convert ConfigV1 to MetaData")
        )
    };

    // let config = MatchComposerConfig::try_from(meta)
    //     .expect("Failed to parse MatchComposerConfig from GameServer metadata");

    log::info!("hub_path : {:?}", args.hub_path);
    log::info!("player_log_root_dir : {:?}", args.player_log_root_dir);

    let addr = SocketAddr::new(args.host.into(), args.port);
    let composer_conf = MatchComposerConfig {
        server: RcssServerConfig {
            control: SocketAddr::new(args.rcss_host, args.rcss_port),
            player: SocketAddr::new(args.rcss_host, args.rcss_player_port),
            trainer: SocketAddr::new(args.rcss_host, args.rcss_trainer_port),
            coach: SocketAddr::new(args.rcss_host, args.rcss_coach_port),
        },
        player_log_root: args.player_log_root_dir.map(|p|log_root.join(p)),
        registry_path: args.hub_path,
        player_spawn_delay: Duration::from_millis(args.player_spawn_delay),
        team_spawn_delay: Duration::from_millis(args.team_spawn_delay),
        concurrent_team_spawn: args.team_spawn_concurrent_en,
    };


    let serv = server::listen(addr, meta, composer_conf).await.expect("Failed to start server");
    serv.await.expect("Server error");

}
