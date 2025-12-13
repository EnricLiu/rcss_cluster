use std::str::FromStr;

use super::{Command, TrainerCommand};
use arcstr::{ArcStr, format, literal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandInit {
    pub version: Option<u8>,
}

impl Command for CommandInit {
    type Kind = TrainerCommand;
    type Ok = CommandInitOk;
    type Error = CommandInitError;

    fn kind(&self) -> Self::Kind {
        TrainerCommand::Init
    }

    fn encode(&self) -> ArcStr {
        if let Some(version) = self.version {
            format!("(init {})", version)
        } else {
            literal!("(init)")
        }
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        tokens.is_empty().then_some(())
    }

    // never error
}

pub type CommandInitOk = ();

#[derive(thiserror::Error, Debug)]
pub enum CommandInitError {}

impl FromStr for CommandInitError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandInitError as FromStr>::Err> {
        Err(())
    }
}
