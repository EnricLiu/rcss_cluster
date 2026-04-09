use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct RcssServerConfig {
    pub control: SocketAddr,
    pub player: SocketAddr,
    pub trainer: SocketAddr,
    pub coach: SocketAddr,
}

