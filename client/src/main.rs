mod agones;
mod room;
mod utils;
mod error;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;

use crate::agones::AgonesClient;
use crate::room::Room;

use error::{Error, Result};


pub struct Proxy {
    pub agones: AgonesClient,
}


impl Proxy {
    pub async fn create_room(&self, udp_port: u16) -> Room {
        let udp_socket = UdpSocket::bind(utils::local_addr(udp_port)).await.expect("Failed to bind UDP socket");

        let udp_port = udp_socket.local_addr().unwrap().port();
        let ws_addr = self.agones.allocate().await.expect("Failed to allocate room");

        Room::listen(udp_port, ws_addr).await.unwrap()
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
