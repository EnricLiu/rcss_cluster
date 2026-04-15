use std::collections::HashMap;

use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Serialize};

use common::types::Side;

use crate::declaration::Unum;
use crate::team::{Error as TeamError, Result as TeamResult};
use super::player::PlayerInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamInfo {
    pub name: String,
    pub side: Side,
    pub status: TeamStatusInfoSerDes,
    pub players: HashMap<Unum, PlayerInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TeamStatusInfoSerDes {
    Idle,
    Starting,
    Running,
    ShuttingDown,
    Aborting {
        reason: String,
    },
    Error {
        reason: String,
    },
}

impl From<TeamStatusInfo> for TeamStatusInfoSerDes {
    fn from(info: TeamStatusInfo) -> Self {
        use TeamStatusInfo::*;
        match info {
            Idle => TeamStatusInfoSerDes::Idle,
            Starting => TeamStatusInfoSerDes::Starting,
            Running => TeamStatusInfoSerDes::Running,
            ShuttingDown => TeamStatusInfoSerDes::ShuttingDown,
            Aborting(reason) => TeamStatusInfoSerDes::Aborting { reason: reason.to_string() },
            Error(reason) => TeamStatusInfoSerDes::Error { reason: reason.to_string() },
        }
    }
}

#[derive(Debug, Clone)]
pub enum TeamStatusInfo {
    Idle,
    Starting,
    Running,
    ShuttingDown,
    Aborting(TeamError),
    Error(TeamError),
}

impl TeamStatusInfo {
    pub fn kind(&self) -> &'static str {
        use TeamStatusInfo::*;
        match self {
            Idle => "idle",
            Starting => "starting",
            Running => "running",
            ShuttingDown => "shutting_down",
            Aborting(_) => "aborting",
            Error(_) => "error",
        }
    }

    pub fn as_err(&self) -> Option<&TeamError> {
        match self {
            TeamStatusInfo::Aborting(e) => Some(e),
            TeamStatusInfo::Error(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_finished(&self) -> bool {
        use TeamStatusInfo::*;
        matches!(self, Idle | Error(_))
    }
    pub fn into_result(self) -> TeamResult<()> {
        use TeamStatusInfo::*;
        match self {
            Idle => Ok(()),
            Error(e) => Err(e),
            _ => Err(TeamError::NotFinished),
        }
    }
}