use std::net::SocketAddr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to open UDP socket for Room at {addr:?}")]
    OpenRoomUdp {
        addr: SocketAddr
    },

    #[error("Failed to open UDP transmission port for client at {addr:?}")]
    OpenClientUdp {
        addr: SocketAddr
    },

    #[error("WebSocket connection error: {source}")]
    WsConnect {
        url: String,
        source: tokio_tungstenite::tungstenite::Error
    }

}

pub type Result<T> = std::result::Result<T, Error>;
