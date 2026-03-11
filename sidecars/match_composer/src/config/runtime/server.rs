use std::net::Ipv4Addr;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerConfig {
    pub host: Ipv4Addr,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: Ipv4Addr::new(127, 0, 0, 1),
            port: 8080,
        }
    }
}
