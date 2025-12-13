mod client;
mod error;
mod process;
mod service;
mod sidecar;
mod test;
mod trainer;

pub use error::{Error, Result};
pub use process::Config as ServerConfig;
pub use sidecar::{Sidecar, SidecarStatus};

pub const RCSS_PROCESS_NAME: &'static str = "rcssserver";
pub const PEER_IP: std::net::IpAddr = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
