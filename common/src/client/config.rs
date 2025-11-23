use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use super::kind::ClientKind;

static DEFAULT_HOST: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
static DEFAULT_PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6000);


#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub name: String,
    pub kind: ClientKind,
    pub host: SocketAddr,
    pub peer: SocketAddr,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            name: "Default Client".to_string(),
            kind: ClientKind::default(),
            host: DEFAULT_HOST,
            peer: DEFAULT_PEER,
        }

    }
}