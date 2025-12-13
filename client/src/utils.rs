use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub fn local_addr(port: u16) -> SocketAddr {
    const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
    SocketAddr::new(LOCALHOST, port)
}
