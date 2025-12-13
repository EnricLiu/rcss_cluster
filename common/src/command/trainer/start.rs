use std::str::FromStr;

use crate::types;
use arcstr::{ArcStr, literal};
use serde::{Deserialize, Serialize};

use super::{Command, CommandAny, TrainerCommand};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandStart;
impl Command for CommandStart {
    type Kind = TrainerCommand;
    type Ok = CommandStartOk;
    type Error = CommandStartError;

    fn kind(&self) -> Self::Kind {
        TrainerCommand::Start
    }

    fn encode(&self) -> ArcStr {
        literal!("(start)")
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        tokens.is_empty().then_some(())
    }

    // never error
}

pub type CommandStartOk = ();

#[derive(thiserror::Error, Debug)]
pub enum CommandStartError {}

impl FromStr for CommandStartError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandStartError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
