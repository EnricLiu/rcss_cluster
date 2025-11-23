use common::client;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Client Closed: {source}")]
    ClientClosed {
        source: client::Error,
    },
    
    #[error("Can not Close Client: {source}")]
    ClientCloseFailed {
        source: client::Error,
    }
}

pub type Result<T> = std::result::Result<T, Error>;
