#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Server is not running, current status: {status:?}")]
    ServerNotRunning { status: crate::ServerStatus },

    #[error("Server is still running, try to set force=true to force respawn.")]
    ServerStillRunningToSpawn,

    #[error("Operation '{op}' timed out.")]
    Timeout { op: &'static str, },

    #[error("Failed to shutdown the process.")]
    ProcessFailedToShutdown,

    #[error("Failed to spawn process: {0}")]
    ProcessSpawnFailed(#[source] process::Error),

    #[error("Failed to send trainer command: {0}")]
    TrainerCommandFailed(String),

    #[error("Status channel closed unexpectedly")]
    StatusChannelClosed,

    #[cfg(feature = "agones")]
    #[error("Failed to connect to Agones SDK: {0}")]
    AgonesSdkFailToConnect(#[source] agones::errors::Error),

    #[cfg(feature = "agones")]
    #[error("Failed to mark Agones SDK as ready: {0}")]
    AgonesSdkReadyFailed(#[source] agones::errors::Error),

    #[cfg(feature = "agones")]
    #[error("Failed to shutdown Agones SDK: {0}")]
    AgonesSdkShutdownFailed(#[source] agones::errors::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
