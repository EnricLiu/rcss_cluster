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

use allocator::declaration;

use crate::metadata::MetaData;
use crate::config::{MatchComposerConfig, RcssServerConfig};

#[tokio::main]
async fn main() {
    unsafe { env::set_var("RUST_LOG", "debug") }
    env_logger::init();
    let args = args::Args::parse();

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
    log::info!("log_root : {:?}", args.log_root);

    let addr = SocketAddr::new(args.host.into(), args.port);
    let composer_conf = MatchComposerConfig {
        server: RcssServerConfig {
            control: SocketAddr::new(args.rcss_host, args.rcss_port),
            player: SocketAddr::new(args.rcss_host, args.rcss_player_port),
            trainer: SocketAddr::new(args.rcss_host, args.rcss_trainer_port),
            coach: SocketAddr::new(args.rcss_host, args.rcss_coach_port),
        },
        log_root: args.log_root,
        registry_path: args.hub_path,
        player_spawn_delay: Duration::from_millis(args.player_spawn_delay),
        team_spawn_delay: Duration::from_millis(args.team_spawn_delay),
        concurrent_team_spawn: args.team_spawn_concurrent_en,
    };


    let serv = server::listen(addr, meta, composer_conf).await.expect("Failed to start server");
    serv.await.expect("Server error");

}
