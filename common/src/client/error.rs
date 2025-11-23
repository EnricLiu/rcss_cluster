use arcstr::ArcStr;
use tokio::sync::mpsc;
use strum_macros::IntoStaticStr;

use crate::client;
use crate::udp::Error as UdpError;

#[derive(thiserror::Error, IntoStaticStr, Debug)]
pub enum Error {
    #[error("Client[{client_name}]: Udp Error: {source}")]
    Udp {
        client_name: String,
        source: UdpError,
    },

    #[error("Client[{client_name}]: Timeout({duration_s} s) waiting client to send an initial message.")]
    TimeoutInitReq {
        client_name: String,
        duration_s: f32,
    },

    #[error("Client[{client_name}]: Timeout({duration_s} s) waiting to recv an initial response.")]
    TimeoutInitResp {
        client_name: String,
        duration_s: f32,
    },

    #[error("Client[{client_name}]: Channel closed unexpectedly.")]
    ChannelClosed {
        client_name: String,
    },

    #[error("Client[{client_name}]: Failed to send to channel, {source}")]
    ChannelSend {
        client_name: String,
        source: mpsc::error::SendError<client::Signal>,
    },

    #[error("Client[{client_name}]: Task Join Error in \"{task_desc}\", {source}")]
    TaskJoin {
        client_name: String,
        task_desc: String,
        source: tokio::task::JoinError,
    },

    #[error("Client[{client_name}]: Already connected.")]
    AlreadyConnected {
        client_name: String,
    },
    
    #[error("Client Not connected, try to call Client::conn first.")]
    NotConnected {
        client: client::Client,
    },
    
    #[error("Client[{client_name}]: Failed to close connection due to Timeout={duration:?}.")]
    CloseTimeout {
        client_name: String,
        duration: std::time::Duration,
    },

    #[error("Client[{client_name}]: Failed to close connection due to Panic.")]
    ClosePanic {
        client_name: String,
    }
    
}

pub type Result<T> = std::result::Result<T, Error>;
