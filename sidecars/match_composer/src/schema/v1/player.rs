use serde::{Deserialize, Serialize};
use crate::schema::Schema;
use crate::schema::v1::utils::pos_in_court;
use super::{PolicyV1, Position};


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlayerV1 {
    pub unum: u8,
    #[serde(default)] // false
    pub goalie: bool,
    #[serde(default="PolicyV1::helios_base")]
    pub policy: PolicyV1,

    #[serde(default)]
    pub init_state: PlayerInitStateV1,

    #[serde(default)]
    pub blocklist: PlayerActionList,
}

impl Schema for PlayerV1 {
    fn verify(&self) -> Result<(), &'static str> {
        if self.unum == 0 {
            return Err("Player unum cannot be 0");
        }
        if self.unum > 12 {
            return Err("Player unum cannot be greater than 12");
        }

        self.policy.verify()?;
        self.init_state.verify()?;
        self.blocklist.verify()?;

        Ok(())
    }
}


/// Default all unset
#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct PlayerInitStateV1 {
    pos: Option<Position>,
    stamina: Option<u16>,
}

impl Schema for PlayerInitStateV1 {
    fn verify(&self) -> Result<(), &'static str> {
        if let Some(pos) = &self.pos {
            pos_in_court(pos.x, pos.y)?;
        }

        Ok(())
    }
}

/// Default for all false
#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct PlayerActionList {
    dash: bool,
    r#catch: bool,
}

impl Schema for PlayerActionList {
    fn verify(&self) -> Result<(), &'static str> {
        Ok(())
    }
}
