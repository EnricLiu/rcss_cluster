use crate::schema::Schema;
use crate::schema::v1::utils::pos_in_court;
use super::{Policy, Position};


#[derive(Clone, Debug)]
pub struct Player {
    unum: u8,
    goalie: bool,
    policy: Policy,

    init_state: PlayerInitState,
    blocklist: PlayerActionList,
}

impl Schema for Player {
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
#[derive(Default, Clone, Debug)]
pub struct PlayerInitState {
    pos: Option<Position>,
    stamina: Option<u16>,
}

impl Schema for PlayerInitState {
    fn verify(&self) -> Result<(), &'static str> {
        if let Some(pos) = &self.pos {
            pos_in_court(pos.x, pos.y)?;
        }

        Ok(())
    }
}

/// Default for all false
#[derive(Default, Clone, Debug)]
pub struct PlayerActionList {
    dash: bool,
    r#catch: bool,
}

impl Schema for PlayerActionList {
    fn verify(&self) -> Result<(), &'static str> {
        Ok(())
    }
}
