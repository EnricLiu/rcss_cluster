use crate::schema::Schema;
pub use super::Player;

#[derive(Clone, Debug)]
pub struct Teams {
    pub allies:     Team,
    pub opponents:  Team,
}

impl Schema for Teams {
    fn verify(&self) -> Result<(), &'static str> {
        if self.allies.side == self.opponents.side {
            return Err("Teams cannot be on the same side")
        }

        if self.allies.name == self.opponents.name {
            return Err("Teams cannot share the same name")
        }

        self.allies.verify()?;
        self.opponents.verify()
    }
}

impl Default for Teams {
    fn default() -> Self {
        let allies = Team {
            name: "AnonymousAllies".to_string(),
            side: TeamSide::Left,
            players: vec![],
        };

        let opponents = Team {
            name: "AnonymousOpponents".to_string(),
            side: TeamSide::Right,
            players: vec![],
        };

        Teams {
            allies,
            opponents,
        }
    }
}


#[derive(Eq, PartialEq, Clone, Debug)]
pub enum TeamSide {
    Left,
    Right
}

#[derive(Clone, Debug)]
pub struct Team {
    name: String,
    side: TeamSide,
    players: Vec<Player>,
}

impl Schema for Team {
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