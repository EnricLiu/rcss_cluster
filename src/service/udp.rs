use std::io::Result;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct UdpConnection(UdpSocket);
impl UdpConnection {
    pub async fn open(host: SocketAddr, peer: SocketAddr) -> Result<Self> {
        let host = UdpSocket::bind(host).await?;
        host.connect(peer).await?;
        Ok(Self(host))
    }

    fn socket(&self) -> &UdpSocket {
        &self.0
    }

    pub async fn send(&self, data: &[u8]) -> Result<()> {
        self.socket().send(data).await?;
        Ok(())
    }

    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        self.socket().recv(buf).await
    }

    pub async fn set_peer(&mut self, peer: SocketAddr) -> Result<()> {
        if self.peer() == peer { return Ok(()) }
        self.socket().connect(peer).await
    }

    pub fn peer(&self) -> SocketAddr {
        self.socket().peer_addr().unwrap()
    }
}