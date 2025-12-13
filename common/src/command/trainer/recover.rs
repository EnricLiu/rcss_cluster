use std::str::FromStr;

use crate::types;
use arcstr::{ArcStr, literal};
use serde::{Deserialize, Serialize};

use super::{Command, CommandAny, TrainerCommand};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandRecover;

impl Command for CommandRecover {
    type Kind = TrainerCommand;
    type Ok = CommandRecoverOk;
    type Error = CommandRecoverError;

    fn kind(&self) -> Self::Kind {
        TrainerCommand::Recover
    }

    fn encode(&self) -> ArcStr {
        literal!("(recover)")
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        tokens.is_empty().then_some(())
    }

    // never error
}

pub type CommandRecoverOk = ();

#[derive(thiserror::Error, Debug)]
pub enum CommandRecoverError {}

impl FromStr for CommandRecoverError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandRecoverError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
