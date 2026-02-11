use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::schema::v1::utils::pos_in_court;

use super::{Schema, Teams, Position};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    teams:  Teams,
    #[serde(default)]
    referee:    Referee,
    #[serde(default)]
    stopping:   StoppingEvent,
    #[serde(default)]
    init_state: GlobalInitState,
    #[serde(default)]
    env:    Option<HashMap<String, String>>
}

impl Schema for Config {
    fn verify(&self) -> Result<(), &'static str> {
        self.teams.verify()?;
        self.referee.verify()?;
        self.stopping.verify()?;
        self.init_state.verify()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Referee {
    enable: bool
}

impl Default for Referee {
    fn default() -> Self {
        Self {
            enable: true
        }
    }
}

impl Schema for Referee {
    fn verify(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct StoppingEvent {
    time_up: Option<u16>,
    goal_l: Option<u8>,
    goal_r: Option<u8>,
}

impl Schema for StoppingEvent {
    fn verify(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct GlobalInitState {
    ball: Option<Position>
}

impl Schema for GlobalInitState {
    fn verify(&self) -> Result<(), &'static str> {
        if let Some(ball) = &self.ball {
            pos_in_court(ball.x, ball.y)?;
        }

        Ok(())
    }
}
