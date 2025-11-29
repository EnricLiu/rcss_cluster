mod process;
mod coach;
mod sidecar;
mod test;

#[cfg(target_os = "windows")]
compile_error!("This program currently not supported on Windows.");

use std::env;
use env_logger;
use log::info;

pub const RCSS_PROCESS_NAME: &'static str = "rcssserver";
pub const PEER_IP: std::net::IpAddr = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);

#[tokio::main]
async fn main() {
    unsafe { env::set_var("RUST_LOG", "trace") };
    env_logger::init();

    let builder = sidecar::Server::spawner().await;
    let mut server = builder.spawn().await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let ret = server.shutdown().await.unwrap();
}

