use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::sync::OnceLock;
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct Operator<'a> {
    peer: SocketAddr,
    socket: &'a UdpSocket,
}

impl<'a> Operator<'a> {
    pub fn new(socket: &'a UdpSocket) -> Self {
        let peer = socket.peer_addr().unwrap();

        Self {
            peer,
            socket,
        }
    }
}