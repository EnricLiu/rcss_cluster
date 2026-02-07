#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("[Coach] Failed to connect, {0:?}")]
    ConnectCoach(crate::client::Error),
    #[error("[Process] Failed to spawn, {0}")]
    SpawnProcess(crate::process::Error),
    #[error("[Coach] Failed to shutdown, {0}")]
    ShutdownCoach(crate::client::Error),
    #[error("[Process] Failed to shutdown, {0}")]
    ShutdownProcess(crate::process::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
