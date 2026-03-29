use std::collections::HashMap;

use serde::ser::SerializeStruct;
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
    Error(TeamError),
}

impl Serialize for TeamStatusInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let status = self.kind();
        let error = self.as_err().map(|e| e.to_string());

        let len = if error.is_some() { 2 } else { 1 };

        let mut state = serializer.serialize_struct("GetResponse", len)?;
        state.serialize_field("status", status)?;
        if let Some(error) = error {
            state.serialize_field("error", &error)?;
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
            Error(_) => "error",
        }
    }

    pub fn as_err(&self) -> Option<&TeamError> {
        match self {
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