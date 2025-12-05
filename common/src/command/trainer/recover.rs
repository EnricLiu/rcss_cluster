use std::str::FromStr;

use arcstr::{ArcStr, literal};
use crate::types;

use super::{Command, CommandAny, TrainerCommand};

pub struct CommandRecover;

impl Command for CommandRecover {
    type Kind = TrainerCommand;
    type Ok = ();
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
