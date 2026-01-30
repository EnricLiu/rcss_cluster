use std::str::FromStr;
use crate::types;
use super::{Command, PlayerCommand};
use arcstr::{ArcStr, format, literal};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CommandInit {
    pub team_name: String,
    pub version: Option<u8>,
    pub is_goalie: bool,
}

impl Command for CommandInit {
    type Kind = PlayerCommand;
    type Ok = CommandInitOk;
    type Error = CommandInitError;

    fn kind(&self) -> Self::Kind {
        PlayerCommand::Init
    }

    fn encode(&self) -> ArcStr {
        let mut ret = String::with_capacity(32);
        ret += &std::format!("(init {}", self.team_name);

        if let Some(version) = self.version {
            ret += &std::format!(" (version {})", version);
        }

        if self.is_goalie {
            ret += " (goalie)"
        }

        ret.into()
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        todo!()
        // tokens.is_empty().then_some(())
    }

    // never error
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandInitOk {
    side: types::Side,
    unum: u8,
    play_mode: types::PlayMode,
}

#[derive(thiserror::Error, Debug)]
pub enum CommandInitError {
    #[error("no more team or player or goalie")]
    NoMoreTeamOrPlayerOrGoalie
}

impl FromStr for CommandInitError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandInitError as FromStr>::Err> {
        match s {
            "no_more_team_or_player_or_goalie" => Ok(CommandInitError::NoMoreTeamOrPlayerOrGoalie),
            _ => Err(())
        }
    }
}
