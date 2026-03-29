use common::types::Side;

use dashmap::DashMap;
use common::errors::{BuilderError, BuilderResult};
use crate::declarations::Unum;
use super::player::Player;

#[derive(Clone, Debug)]
pub struct Team {
    pub name: String,
    pub side: Side,
    pub players: DashMap<Unum, Player>,
}

impl Team {
    pub fn builder() -> TeamBuilder {
        TeamBuilder::default()
    }
}


#[derive(Debug, Default)]
pub struct TeamBuilder {
    pub name: Option<String>,
    pub side: Option<Side>,
    pub players: DashMap<Unum, Player>,
}

impl TeamBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }
    
    pub fn with_side(&mut self, side: Side) -> &mut Self {
        self.side = Some(side);
        self
    }
    
    pub fn add_player(&mut self, player: Player) -> &mut Self {
        self.players.insert(player.unum, player);
        self
    }
    
    pub fn build(&self) -> BuilderResult<Team> {
        let name = self.name.clone().ok_or(BuilderError::MissingField { field: "name" })?;
        let side = self.side.ok_or(BuilderError::MissingField { field: "side" })?;
        if self.players.is_empty() {
            return Err(BuilderError::MissingField { field: "players" });
        }
        
        Ok(Team {
            name,
            side,
            players: self.players.clone(),
        })
    }
}
