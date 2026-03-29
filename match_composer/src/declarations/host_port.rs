use std::net::Ipv4Addr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostPort {
    pub host: Ipv4Addr,
    pub port: u16,
}
