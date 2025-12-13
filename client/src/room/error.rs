use crate::room::RoomConfig;
use std::net::SocketAddr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to open UDP server socket for Room {room:?}")]
    OpenRoomUdp { room: RoomConfig },

    #[error("Failed to open UDP transmission port for client at {addr:?}, room = {room:?}")]
    OpenClientUdp { room: RoomConfig, addr: SocketAddr },

    #[error("WebSocket connection error: {source}, room = {room:?}")]
    WsConnect {
        room: RoomConfig,
        source: tokio_tungstenite::tungstenite::Error,
    },

    #[error("Failed to send message to WebSocket")]
    WsSendFailed {
        room: RoomConfig,
        source: tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>,
    },

    #[error("Failed to send UDP message")]
    UdpSendFailed {
        room: RoomConfig,
        source: common::udp::Error,
    },

    #[error("ProxyServer connection not initialized")]
    ProxyNotInitialized { room: RoomConfig },

    #[error("Websocket Connector task stopped")]
    WsConnectorDown { room: RoomConfig },
}

pub type Result<T> = std::result::Result<T, Error>;
