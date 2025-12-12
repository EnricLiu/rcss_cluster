use std::net::SocketAddr;
use log::trace;
use tokio::net::UdpSocket;

use super::{Result, Error};

#[derive(Debug)]
pub struct UdpConnection(UdpSocket);
impl UdpConnection {
    pub async fn open(host: SocketAddr, peer: SocketAddr) -> Result<Self> {
        let host = UdpSocket::bind(host).await
            .map_err(|e| Error::Open { host, source: e })?;

        host.connect(peer).await
            .map_err(|e| Error::Connect { peer, source: e })?;
        
        Ok(Self(host))
    }
    
    pub async fn bind(host: SocketAddr) -> Result<Self> {
        let host = UdpSocket::bind(host).await
            .map_err(|e| Error::Open { host, source: e })?;
        
        Ok(Self(host))
    }

    fn socket(&self) -> &UdpSocket {
        &self.0
    }
    
    pub async fn send_and_conn_new_recv(
        &self,
        data: &[u8], buf: &mut [u8],
        peer: SocketAddr,
    ) -> Result<usize> {
        self.socket().send_to(data, peer).await
            .map_err(|e| Error::Send { source: e })?;
        self.recv_set_peer(buf).await
    }

    pub async fn send(&self, data: &[u8]) -> Result<()> {
        self.socket().send(data).await
            .map_err(|e| Error::Send { source: e })?;
        Ok(())
    }

    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        self.socket().recv(buf).await
            .map_err(|e| Error::Recv { source: e })
    }

    pub async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.socket().recv_from(buf).await
            .map_err(|e| Error::Recv { source: e })
    }

    pub async fn recv_set_peer(&self, buf: &mut [u8]) -> Result<usize> {
        let (len, peer) = self.recv_from(buf).await?;
        self.set_peer(peer).await?;
        trace!("UDP[{:?}]: recv len={len}, set peer to [{peer}]", self.socket().local_addr());
        Ok(len)
    }

    pub async fn set_peer(&self, peer: SocketAddr) -> Result<()> {
        self.socket().connect(peer).await
            .map_err(|e| Error::Connect { peer, source: e })
    }
    
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket().local_addr()
    }

    pub fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket().peer_addr()
    }
}
