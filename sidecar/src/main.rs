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
    unsafe { env::set_var("RUST_LOG", "info") };
    env_logger::init();
    
    let builder = process::ServerProcess::spawner("rcssserver").await;
    let process = builder.spawn().await.unwrap();
    info!("Process running, pid = {:?}", process.pid());
    
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    
    let ret = process.shutdown().await.unwrap();
    info!("Process terminated, ret code = {ret}")
}
