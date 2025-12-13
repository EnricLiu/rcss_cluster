use std::str::FromStr;

use super::{Command, TrainerCommand};
use arcstr::{ArcStr, literal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandLook;
impl Command for CommandLook {
    type Kind = TrainerCommand;
    type Ok = CommandLookOk;
    type Error = CommandLookError;

    fn kind(&self) -> Self::Kind {
        TrainerCommand::Look
    }

    fn encode(&self) -> ArcStr {
        literal!("(look)")
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        todo!("really complex to implement")
    }

    // never error
}

pub type CommandLookOk = ();

#[derive(thiserror::Error, Debug)]
pub enum CommandLookError {}

impl FromStr for CommandLookError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandLookError as FromStr>::Err> {
        Err(())
    }
}
