#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Server is not running, current status: {status:?}")]
    ServerNotRunning {
        status: crate::sidecar::SidecarStatus,
    },

    #[error("Server is still running, try restart by setting force = true.")]
    ServerStillRunningToRestart,
}

pub type Result<T> = std::result::Result<T, Error>;
