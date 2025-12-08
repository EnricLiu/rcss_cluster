use std::str::FromStr;

use arcstr::{ArcStr, format};
use serde::{Deserialize, Serialize};
use crate::command::trainer::ear::CommandEarOk;
use crate::types::EyeMode;

use super::{Command, CommandAny, TrainerCommand};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandEye {
    pub mode: EyeMode,
}

impl Command for CommandEye {
    type Kind = TrainerCommand;
    type Ok = CommandEarOk;
    type Error = CommandEyeError;

    fn kind(&self) -> Self::Kind {
        TrainerCommand::Eye
    }
    fn encode(&self) -> ArcStr {
        format!("(eye {})", self.mode.encode())
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        if tokens.len() != 1 { return None }
        let eye_mode = tokens[0].parse().ok();
        eye_mode.map(|mode| CommandEarOk { mode })
    }

    fn parse_ret_err(tokens: &[&str]) -> Option<Self::Error> {
        if tokens.len() != 1 { return None }
        let tokens = tokens.join(" ");
        if tokens.is_empty() { return None }

        tokens.parse().ok()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandEyeOk {
    pub mode: EyeMode,
}

#[derive(thiserror::Error, Debug)]
pub enum CommandEyeError {
    #[error("MODE did not match on or off.")]
    IllegalMode,
    #[error("The MODE argument was omitted.")]
    IllegalCommandForm,
}

impl FromStr for CommandEyeError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandEyeError as FromStr>::Err> {
        match s {
            "illegal_mode" => Ok(Self::IllegalMode),
            "illegal_command_form" => Ok(Self::IllegalCommandForm),
            _ => Err(()),
        }
    }
}
