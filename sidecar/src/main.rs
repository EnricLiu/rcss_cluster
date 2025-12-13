#[cfg(target_os = "windows")]
compile_error!("This program currently not supported on Windows.");

mod client;
mod process;
mod service;
mod test;
mod trainer;

use env_logger;
use std::env;

pub const RCSS_PROCESS_NAME: &'static str = "rcssserver";
pub const PEER_IP: std::net::IpAddr = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);

#[tokio::main]
async fn main() {
    unsafe { env::set_var("RUST_LOG", "trace") };
    env_logger::init();
}
