use serde::Serialize;

#[derive(Serialize, Copy, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ServerStatus {
    Uninitialized,
    Idle,
    Simulating,
    Finished,
    Shutdown,
}

impl ServerStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, ServerStatus::Simulating)
    }
    
    pub fn is_healthy(&self) -> bool {
        matches!(self, ServerStatus::Simulating | ServerStatus::Idle)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, ServerStatus::Idle)
    }

    pub fn is_initialized(&self) -> bool {
        !matches!(self, ServerStatus::Uninitialized)
    }
    
    pub fn is_finished(&self) -> bool {
        matches!(self, ServerStatus::Finished)
    }
}

impl From<ServerStatus> for u8 {
    fn from(val: ServerStatus) -> Self {
        val as u8
    }
}

impl TryFrom<u8> for ServerStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ServerStatus::Uninitialized),
            1 => Ok(ServerStatus::Idle),
            2 => Ok(ServerStatus::Simulating),
            3 => Ok(ServerStatus::Finished),
            4 => Ok(ServerStatus::Shutdown),
            _ => Err(()),
        }
    }
}
