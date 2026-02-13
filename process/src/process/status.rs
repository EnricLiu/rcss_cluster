use std::sync::Arc;
use std::process::ExitStatus;

use tokio::sync::RwLock;

use common::utils::ringbuf::OverwriteRB;

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
    pub fn is_finished(&self) -> bool {
        match self {
            ProcessStatusKind::Returned(_) => true,
            ProcessStatusKind::Dead(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            ProcessStatusKind::Returned(status) => !status.success(),
            ProcessStatusKind::Dead(_) => true,
            _ => false,
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
