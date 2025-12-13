use std::str::FromStr;

use crate::types;
use arcstr::{ArcStr, format};
use serde::{Deserialize, Serialize};

use super::{Command, CommandAny, TrainerCommand};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandMove {
    pub todo: (),
}

impl Command for CommandMove {
    type Kind = TrainerCommand;
    type Ok = CommandMoveOk;
    type Error = CommandMoveError;

    fn kind(&self) -> Self::Kind {
        TrainerCommand::Move
    }

    fn encode(&self) -> ArcStr {
        todo!()
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        tokens.is_empty().then_some(())
    }

    fn parse_ret_err(tokens: &[&str]) -> Option<Self::Error> {
        todo!("really complex too")
    }
}

pub type CommandMoveOk = ();

#[derive(thiserror::Error, Debug)]
pub enum CommandMoveError {
    #[error("The specified mode was not valid.")]
    IllegalMode,
    #[error("The PLAY_MODE argument was omitted")]
    IllegalCommandForm,
}

impl FromStr for CommandMoveError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandMoveError as FromStr>::Err> {
        match s {
            "illegal_mode" => Ok(Self::IllegalMode),
            "illegal_command_form" => Ok(Self::IllegalCommandForm),
            _ => Err(()),
        }
    }
}
