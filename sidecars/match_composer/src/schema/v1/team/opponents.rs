use serde::{Deserialize, Serialize};
use crate::schema::v1::{Schema, Player, Team};
use super::TeamSide;


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OpponentsTeam {
    pub name: String,
    #[serde(default="TeamSide::opponents")]
    pub side: TeamSide,
    pub players: Vec<Player>,
}

impl Schema for OpponentsTeam {
    fn verify(&self) -> Result<(), &'static str> {
        if self.name.is_empty() {
            return Err("Team name cannot be empty")
        }

        if !self.name.is_ascii() {
            return Err("Team name cannot contain non-ASCII characters")
        }

        if self.name.len() > 16 {
            return Err("Team name cannot be longer than 16 characters")
        }

        if self.players.len() > 11 {
            return Err("Team cannot have more than 11 players")
        }

        for player in self.players.iter() {
            player.verify()?;
        }

        Ok(())
    }
}

impl Into<Team> for OpponentsTeam {
    fn into(self) -> Team {
        Team {
            name: self.name,
            side: self.side,
            players: self.players,
        }
    }
}
