use arcstr::ArcStr;
use common::client;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Client Closed: {source}")]
    ClientClosed { source: client::Error },

    #[error("Can not Close Client: {source}")]
    ClientCloseFailed { source: client::Error },

    #[error("Failed to send command to server")]
    CommandSendFailed,

    #[error("Failed to receive command response")]
    CommandReceiveFailed,

    #[error("Command response type mismatch")]
    CommandResponseTypeMismatch,

    #[error("Sender channel has been closed")]
    SenderClosed,

    #[error("CallResolver not initialized")]
    ResolverNotInitialized,

    #[error("CallResolver is not singleton")]
    ResolverNotSingleton,

    #[error("CallResolver timeout")]
    CallElapsed {
        kind: ArcStr,
    },
    
    #[error("Rcssserver response error toward '({kind})': {msg}")]
    RcssErrorCall {
        kind: ArcStr,
        msg: ArcStr,
    }
}

pub type Result<T> = std::result::Result<T, Error>;
