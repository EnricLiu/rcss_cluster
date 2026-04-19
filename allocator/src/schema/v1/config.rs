use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::schema::v1::utils::pos_in_court;

use super::{Schema, TeamsV1, Position};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ConfigV1 {
    #[serde(default)]
    pub log: bool,
    pub teams: TeamsV1,
    #[serde(default)]
    pub referee: RefereeV1,
    #[serde(default)]
    pub stopping: StoppingEventV1,
    #[serde(default)]
    pub init_state: GlobalInitStateV1,
    #[serde(default)]
    pub env:    Option<HashMap<String, String>>
}

impl Schema for ConfigV1 {
    fn verify(&self) -> Result<(), &'static str> {
        self.teams.verify()?;
        self.referee.verify()?;
        self.stopping.verify()?;
        self.init_state.verify()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RefereeV1 {
    pub enable: bool
}

impl Default for RefereeV1 {
    fn default() -> Self {
        Self {
            enable: true
        }
    }
}

impl Schema for RefereeV1 {
    fn verify(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct StoppingEventV1 {
    pub time_up: Option<u16>,
    pub goal_l: Option<u8>,
    pub goal_r: Option<u8>,
}

impl Schema for StoppingEventV1 {
    fn verify(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct GlobalInitStateV1 {
    pub ball: Option<Position>
}

impl Schema for GlobalInitStateV1 {
    fn verify(&self) -> Result<(), &'static str> {
        if let Some(ball) = &self.ball {
            pos_in_court(ball.x, ball.y)?;
        }

        Ok(())
    }
}
