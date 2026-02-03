use axum::extract::ws::Message;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("WebSocket send error: {source}")]
    Channel {
        source: tokio::sync::mpsc::error::SendError<Message>,
    },

    #[error("[WS] Failed to send to Client[{client_id}]: {source}")]
    WsSend {
        client_id: uuid::Uuid,
        source: axum::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
