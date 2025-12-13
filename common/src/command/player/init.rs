use std::str::FromStr;

use super::{Command, CommandAny, PlayerCommand};
use arcstr::{ArcStr, format, literal};

#[derive(Debug)]
pub struct CommandInit {
    pub version: Option<u8>,
}

impl Command for CommandInit {
    type Kind = PlayerCommand;
    type Ok = ();
    type Error = CommandInitError;

    fn kind(&self) -> Self::Kind {
        PlayerCommand::Init
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

#[derive(thiserror::Error, Debug)]
pub enum CommandInitError {}

impl FromStr for CommandInitError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandInitError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
