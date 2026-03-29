use serde::Serialize;
use chrono::{DateTime, Utc};

use crate::declarations::HostPort;
use super::team::TeamInfo;

#[derive(Serialize, Debug, Clone)]
pub struct GameInfo {
    pub rcss: HostPort,
    pub status: GameStatusInfo,
    pub team_l: TeamInfo,
    pub team_r: TeamInfo,
}

#[derive(Serialize, Debug, Clone)]
pub enum GameStatusInfo {
    Idle,
    Running {
        started_at: DateTime<Utc>,
    },
    Finished {
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
    },
    Terminated {
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        reason: String,
    },
}