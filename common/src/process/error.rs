use nix::sys::signal::Signal;
use std::process::ExitStatus;
use tokio::io;

#[derive(thiserror::Error, Debug)]
pub enum ProcessError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    SignalSend(#[from] tokio::sync::mpsc::error::SendError<Signal>),

    #[error(transparent)]
    ProcessJoinTimeout(#[from] tokio::time::error::Elapsed),

    #[error(transparent)]
    ProcessJoin(#[from] tokio::task::JoinError),

    #[error("FATAL: Can not wind up process!")]
    FatalProcessWindingUp {
        pid: Option<u32>,
        signal: Signal,
        error: io::Error,
    },

    #[error("The child process is already completed.")]
    ChildAlreadyCompleted(ExitStatus),

    #[error("The child process is running without PID?????")]
    ChildRunningWithoutPid,

    #[error("FATAL: The child process is without a PID and can not be tracked, due to: {0}")]
    ChildUntrackableWithoutPid(io::Error),

    #[error("The child process returned with status: {0}")]
    ChildReturned(ExitStatus),

    #[error("The child process[pid={pid:?}] is dead with error: {error}")]
    ChildDead { pid: Option<u32>, error: String },

    #[error("Timeout waiting for child process to be ready")]
    TimeoutWaitingReady,

    #[error("The child process is not ready anyway :(")]
    ChildNotReady,

    #[error("Failed to capture stdout from child process")]
    StdoutCaptureFailed,

    #[error("Failed to capture stderr from child process")]
    StderrCaptureFailed,
}

pub type Result<T> = std::result::Result<T, ProcessError>;
