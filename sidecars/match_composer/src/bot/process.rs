use tokio::sync::watch;
use tokio::process::Command;

use common::process::{Process, ProcessStatus};

#[derive(Debug)]
pub struct BotProcess {
    pub process: Process,
}

impl BotProcess {
    pub(crate) fn spawn(mut command: Command) -> Result<Self> {
        let child = command.spawn().map_err(Error::ChildFailedSpawn)?;
        let process = Process::new(child)?;
        
        Ok(Self {
            process
        })
    }
    
    pub(crate) fn status_watch(&self) -> watch::Receiver<ProcessStatus> {
        self.process.status_watch()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Process(#[from] common::process::ProcessError),
    
    #[error("Failed to spawn child process, {0}")]
    ChildFailedSpawn(#[source] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
