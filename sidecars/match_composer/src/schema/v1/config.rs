use std::collections::HashMap;
use crate::schema::v1::utils::pos_in_court;
use super::{Schema, Teams, Position};

#[derive(Default, Clone, Debug)]
pub struct Config {
    referee:    RefereeConfig,
    stopping:   StoppingEvent,
    init_state: GlobalInitState,
    teams:  Teams,
    env:    HashMap<String, String>
}

impl Schema for Config {
    fn verify(&self) -> Result<(), &'static str> {
        self.teams.verify()?;
        self.referee.verify()?;
        self.stopping.verify()?;
        self.init_state.verify()
    }
}

#[derive(Clone, Debug)]
pub struct RefereeConfig {
    enable: bool
}

impl Default for RefereeConfig {
    fn default() -> Self {
        Self {
            enable: true
        }
    }
}

impl Schema for RefereeConfig {
    fn verify(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Default, Clone, Debug)]
pub struct StoppingEvent {
    timeup: Option<u16>,
    goal_l: Option<u8>,
    goal_r: Option<u8>,
}

impl Schema for StoppingEvent {
    fn verify(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Default, Clone, Debug)]
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

