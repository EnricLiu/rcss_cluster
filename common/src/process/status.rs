use std::sync::Arc;
use std::process::ExitStatus;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::utils::ringbuf::OverwriteRB;

#[derive(Clone, Debug)]
pub struct ProcessStatus<const OUT: usize = 32, const ERR: usize = 32> {
    pub kind: ProcessStatusKind,
    pub stdout: Arc<RwLock<OverwriteRB<String, OUT>>>,
    pub stderr: Arc<RwLock<OverwriteRB<String, ERR>>>,
}

impl<const OUT: usize, const ERR: usize> ProcessStatus<OUT, ERR> {
    pub fn new() -> Self {
        ProcessStatus {
            kind: ProcessStatusKind::Init,
            stdout: Arc::new(RwLock::new(OverwriteRB::new())),
            stderr: Arc::new(RwLock::new(OverwriteRB::new())),
        }
    }

    pub fn init() -> Self {
        Self::new()
    }

    pub fn as_init(&mut self) {
        self.kind = ProcessStatusKind::Init;
    }
    pub fn as_booting(&mut self) {
        self.kind = ProcessStatusKind::Booting;
    }
    pub fn as_running(&mut self) {
        self.kind = ProcessStatusKind::Running;
    }

    pub fn as_returned(&mut self, status: ExitStatus) {
        self.kind = ProcessStatusKind::Returned(status);
    }

    pub fn as_dead(&mut self, reason: String) {
        self.kind = ProcessStatusKind::Dead(reason);
    }

    pub fn is_ready(&self) -> bool {
        self.kind.is_ready()
    }

    pub fn is_finished(&self) -> bool {
        self.kind.is_finished()
    }

    pub fn is_err(&self) -> bool {
        self.kind.is_err()
    }

    pub fn status(&self) -> ProcessStatusKind {
        self.kind.clone()
    }

    pub fn stdout(&self) -> Arc<RwLock<OverwriteRB<String, OUT>>> {
        self.stdout.clone()
    }

    pub async fn stdout_logs(&self) -> Vec<String> {
        self.stdout().read().await.to_vec()
    }

    pub fn stderr(&self) -> Arc<RwLock<OverwriteRB<String, ERR>>> {
        self.stderr.clone()
    }

    pub async fn stderr_logs(&self) -> Vec<String> {
        self.stderr().read().await.to_vec()
    }
}

#[derive(Clone, Debug)]
pub enum ProcessStatusKind {
    Init,
    Booting,
    Running,
    Returned(ExitStatus),
    Dead(String),
}

impl ProcessStatusKind {
    pub fn name(&self) -> &'static str {
        match self {
            ProcessStatusKind::Init => "init",
            ProcessStatusKind::Booting => "booting",
            ProcessStatusKind::Running => "running",
            ProcessStatusKind::Returned(_) => "returned",
            ProcessStatusKind::Dead(_) => "dead",
        }
    }
    
    pub fn is_finished(&self) -> bool {
        match self {
            ProcessStatusKind::Returned(_) => true,
            ProcessStatusKind::Dead(_) => true,
            _ => false,
        }
    }

    pub fn is_booting(&self) -> bool {
        matches!(self, ProcessStatusKind::Booting)
    }

    pub fn is_running(&self) -> bool {
        matches!(self, ProcessStatusKind::Running)
    }

    pub fn is_err(&self) -> bool {
        match self {
            ProcessStatusKind::Returned(status) => !status.success(),
            ProcessStatusKind::Dead(_) => true,
            _ => false,
        }
    }
    
    pub fn is_success(&self) -> bool {
        match self {
            ProcessStatusKind::Returned(status) => status.success(),
            _ => false,
        }
    }
    
    pub fn as_err(&self) -> Option<String> {
        match self {
            ProcessStatusKind::Returned(status) => (!status.success()).then(||
                format!("Process exited with status code: {}", status.code().unwrap_or(-1))
            ),
            ProcessStatusKind::Dead(desc) => Some(desc.clone()),
            _ => None,
        }
    }

    pub fn as_finished(&self) -> Option<String> {
        match self {
            ProcessStatusKind::Returned(status) => {
                (!status.success()).then(||
                    format!("Process exited with status code: {}", status.code().unwrap_or(-1))
                ).or_else(|| Some("Process exited with no error.".to_string()))
            },
            ProcessStatusKind::Dead(desc) => Some(desc.clone()),
            _ => None,
        }
    }

    pub fn is_ready(&self) -> bool {
        match self {
            ProcessStatusKind::Running => true,
            _ => false,
        }
    }

    pub fn ord(&self) -> usize {
        match self {
            ProcessStatusKind::Init => 0,
            ProcessStatusKind::Booting => 1,
            ProcessStatusKind::Running => 2,
            ProcessStatusKind::Returned(_) => 3,
            ProcessStatusKind::Dead(_) => 4,
        }
    }
}

impl Serialize for ProcessStatusKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_map(None)?;
        
        ser.serialize_entry("status", self.name())?;
        if let Some(error) = self.as_err() {
            ser.serialize_entry("error", &error)?;
        }

        ser.end()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ProcessStatusKindSerDes {
    Init,
    Booting,
    Running,
    Returned { code: i32, success: bool },
    Dead { reason: String },
}

impl From<ProcessStatusKind> for ProcessStatusKindSerDes {
    fn from(kind: ProcessStatusKind) -> Self {
        match kind {
            ProcessStatusKind::Init => ProcessStatusKindSerDes::Init,
            ProcessStatusKind::Booting => ProcessStatusKindSerDes::Booting,
            ProcessStatusKind::Running => ProcessStatusKindSerDes::Running,
            ProcessStatusKind::Returned(status) => ProcessStatusKindSerDes::Returned { 
                code: status.code().unwrap_or(-1), 
                success: status.success(),
            },
            ProcessStatusKind::Dead(reason) => ProcessStatusKindSerDes::Dead { reason },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessStatusSerDes {
    #[serde(flatten)]
    pub kind: ProcessStatusKindSerDes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessStatusSerDesVerbose {
    #[serde(flatten)]
    pub status: ProcessStatusSerDes,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}


impl<const OUT: usize, const ERR: usize> ProcessStatus<OUT, ERR> {
    pub fn serialize(&self) -> ProcessStatusSerDes {
        ProcessStatusSerDes {
            kind: self.kind.clone().into(),
        }
    }

    pub async fn serialize_verbose(&self) -> ProcessStatusSerDesVerbose {
        ProcessStatusSerDesVerbose {
            status: self.serialize(),
            stdout: self.stdout_logs().await,
            stderr: self.stderr_logs().await,
        }
    }
}