use std::collections::HashMap;

use serde::ser::{SerializeMap, SerializeStruct};
use serde::Serialize;

use common::types::Side;

use crate::declarations::Unum;
use crate::team::{Error as TeamError, Result as TeamResult};
use super::player::PlayerInfo;

#[derive(Serialize, Debug, Clone)]
pub struct TeamInfo {
    pub name: String,
    pub side: Side,
    pub status: TeamStatusInfo,
    pub players: HashMap<Unum, PlayerInfo>,
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

impl Serialize for TeamStatusInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use TeamStatusInfo::*;
        let status = self.kind();

        let mut state = serializer.serialize_map(None)?;
        state.serialize_entry("status", status)?;

        if let Aborting(reason) = self {
            state.serialize_entry("reason", &reason.to_string())?;
        }
        
        state.end()
    }
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