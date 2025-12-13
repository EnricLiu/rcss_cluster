use std::str::FromStr;

use crate::types::EarMode;
use arcstr::{ArcStr, format};
use serde::{Deserialize, Serialize};

use super::{Command, CommandAny, TrainerCommand};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandEar {
    pub mode: EarMode,
}

impl Command for CommandEar {
    type Kind = TrainerCommand;
    type Ok = EarMode;
    type Error = CommandEarError;

    fn kind(&self) -> Self::Kind {
        TrainerCommand::Ear
    }
    fn encode(&self) -> ArcStr {
        format!("(ear {})", self.mode.encode())
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        if tokens.len() != 1 {
            return None;
        }
        tokens[0].parse().ok()
    }

    fn parse_ret_err(tokens: &[&str]) -> Option<Self::Error> {
        if tokens.len() != 1 {
            return None;
        }
        tokens[0].parse().ok()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandEarOk {
    pub mode: EarMode,
}

#[derive(thiserror::Error, Debug)]
pub enum CommandEarError {
    #[error("MODE did not match on or off.")]
    IllegalMode,
    #[error("The MODE argument was omitted.")]
    IllegalCommandForm,
}

impl FromStr for CommandEarError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandEarError as FromStr>::Err> {
        match s {
            "illegal_mode" => Ok(Self::IllegalMode),
            "illegal_command_form" => Ok(Self::IllegalCommandForm),
            _ => Err(()),
        }
    }
}
