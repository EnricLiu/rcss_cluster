use std::str::FromStr;

use arcstr::{ArcStr, literal};
use super::{Command, CommandAny, TrainerCommand};

pub struct CommandLook;
impl Command for CommandLook {
    type Kind = TrainerCommand;
    type Ok = ();
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

#[derive(thiserror::Error, Debug)]
pub enum CommandLookError {}

impl FromStr for CommandLookError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandLookError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
