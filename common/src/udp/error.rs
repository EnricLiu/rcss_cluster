use std::net::SocketAddr;
use strum_macros::IntoStaticStr;

#[derive(thiserror::Error, IntoStaticStr, Debug)]
pub enum Error {
    #[error("Failed to bind UDP socket on \"{host}\"")]
    Open {
        host: SocketAddr,
        source: std::io::Error,
    },

    #[error("Failed to connect \"{peer}\"")]
    Connect {
        peer: SocketAddr,
        source: std::io::Error,
    },

    #[error("Failed to send")]
    Send {
        source: std::io::Error,
    },

    #[error("Failed to receive")]
    Recv {
        source: std::io::Error,
    },
    
    #[error("Timeout to receive")]
    TimeoutRecv,
}

pub type Result<T> = std::result::Result<T, Error>;
