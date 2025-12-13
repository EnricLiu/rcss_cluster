use std::process::ExitStatus;

#[derive(Clone, Debug)]
pub enum ProcessStatus {
    Init,
    Booting,
    Running,
    Returned(ExitStatus),
    Dead(String),
}

impl ProcessStatus {
    pub fn is_finished(&self) -> bool {
        match self {
            ProcessStatus::Returned(status) => !status.success(),
            ProcessStatus::Dead(_) => true,
            _ => false,
        }
    }

    pub fn is_ready(&self) -> bool {
        match self {
            ProcessStatus::Running => true,
            _ => false,
        }
    }

    pub fn ord(&self) -> usize {
        match self {
            ProcessStatus::Init => 0,
            ProcessStatus::Booting => 1,
            ProcessStatus::Running => 2,
            ProcessStatus::Returned(_) => 3,
            ProcessStatus::Dead(_) => 4,
        }
    }
}
