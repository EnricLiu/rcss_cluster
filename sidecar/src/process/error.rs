use std::process::ExitStatus;
use nix::sys::signal::Signal;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to start server: system process limit is reached. source: {0}")]
    MaxProcessReached(#[source] std::io::Error),

    #[error(transparent)]
    Io(std::io::Error),

    #[error(transparent)]
    SignalSend(tokio::sync::mpsc::error::SendError<Signal>),

    #[error(transparent)]
    ProcessJoinTimeout(tokio::time::error::Elapsed),

    #[error(transparent)]
    ProcessJoin(tokio::task::JoinError),

    #[error("FATAL: Can not wind up process!")]
    FatalProcessWindingUp {
        pid: Option<u32>,
        signal: Signal,
        error: std::io::Error,
    },
    
    #[error("The child process is already completed.")]
    ChildAlreadyCompleted(ExitStatus),

    #[error("The child process is running without PID?????")]
    ChildRunningWithoutPid,

    #[error("FATAL: The child process is without a PID and can not be tracked, due to: {0}")]
    ChildUntrackableWithoutPid(tokio::io::Error),

    #[error("The child process returned with status: {0}")]
    ChildReturned(ExitStatus),

    #[error("The child process[pid={pid:?}] is dead with error: {error}")]
    ChildDead {
        pid: Option<u32>,
        error: String,
    },

    #[error("Timeout waiting for child process to be ready")]
    TimeoutWaitingReady,

    #[error("The child process is not ready anyway :(")]
    ChildNotReady,
}

pub type Result<T> = std::result::Result<T, Error>;