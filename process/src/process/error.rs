use common::process::ProcessError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to start server: system process limit is reached. source: {0}")]
    MaxProcessReached(#[source] std::io::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Process(#[from] ProcessError),
}

pub type Result<T> = std::result::Result<T, Error>;
