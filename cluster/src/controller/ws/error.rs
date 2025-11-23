use axum::extract::ws::Message;

#[derive(snafu::Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("WebSocket send error: {source}"))]
    Channel {
        source: tokio::sync::mpsc::error::SendError<Message>,
        backtrace: std::backtrace::Backtrace
    },
    
    #[snafu(display("[WS] Failed to send to Client[{client_id}]: {source}"))]
    WsSend {
        client_id: uuid::Uuid,
        source: axum::Error,
        backtrace: std::backtrace::Backtrace
    },
    
    #[snafu(display("Client[{client_id}] Failed to send data through udp: {source}"))]
    ClientSend {
        client_id: uuid::Uuid,
        source: crate::service::client::Error,
        backtrace: std::backtrace::Backtrace
    }
}

pub type Result<T> = std::result::Result<T, Error>;