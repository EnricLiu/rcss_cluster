use std::net::{IpAddr, SocketAddr};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostPort {
    pub host: IpAddr,
    pub port: u16,
}

impl From<SocketAddr> for HostPort {
    fn from(addr: SocketAddr) -> Self {
        HostPort {
            host: addr.ip(),
            port: addr.port(),
        }
    }
}

impl From<HostPort> for SocketAddr {
    fn from(host_port: HostPort) -> Self {
        SocketAddr::new(host_port.host.into(), host_port.port)
    }

}
