mod policy;
mod server;
pub mod composer;
pub mod team;

mod agones;
mod player;
mod declarations;
mod model;
mod args;
pub mod info;

use std::env;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use clap::Parser;
use crate::agones::AgonesMetaData;
use crate::composer::MatchComposerConfig;
use crate::declarations::HostPort;

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
        AgonesMetaData::try_from(meta).unwrap()
    } else {
        serde_json::from_str::<AgonesMetaData>(
            &std::fs::read_to_string(args.file.unwrap())
                .expect("Failed to read config file")
        ).expect("Failed to parse ConfigV1 from config file")
    };

    // let config = MatchComposerConfig::try_from(meta)
    //     .expect("Failed to parse MatchComposerConfig from GameServer metadata");

    log::info!("hub_path : {:?}", args.hub_path);
    log::info!("log_root : {:?}", args.log_root);

    let addr = SocketAddr::new(args.host.into(), args.port);
    let composer_conf = MatchComposerConfig {
        server: HostPort {
            host: args.rcss_host,
            port: args.rcss_port,
        },
        log_root: args.log_root,
        registry_path: Default::default(),
    };


    let serv = server::listen(addr, meta, composer_conf).await.expect("Failed to start server");
    serv.await.expect("Server error");

}
