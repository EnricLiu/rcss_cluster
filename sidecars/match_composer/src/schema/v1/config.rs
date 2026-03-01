use std::collections::HashMap;
use std::net::Ipv4Addr;
use serde::{Deserialize, Serialize};

use crate::schema::v1::utils::pos_in_court;

use super::{Schema, TeamsV1, Position};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ConfigV1 {
    #[serde(default = "default_host")]
    pub host: Ipv4Addr,
    #[serde(default = "default_port")]
    pub port: u16,
    
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

const fn default_host() -> Ipv4Addr {
    Ipv4Addr::new(127, 0, 0, 1)
}

const fn default_port() -> u16 {
    6000
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
    enable: bool
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
    time_up: Option<u16>,
    goal_l: Option<u8>,
    goal_r: Option<u8>,
}

impl Schema for StoppingEventV1 {
    fn verify(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct GlobalInitStateV1 {
    ball: Option<Position>
}

impl Schema for GlobalInitStateV1 {
    fn verify(&self) -> Result<(), &'static str> {
        if let Some(ball) = &self.ball {
            pos_in_court(ball.x, ball.y)?;
        }

        Ok(())
    }
}
