pub mod init;

pub use init::CommandInit;

use arcstr::{ArcStr, format, literal};
use std::any::Any;

use super::{Command, CommandAny};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum PlayerCommand {
    Init,
}

impl CommandAny for PlayerCommand {
    fn encode(&self) -> ArcStr {
        match self {
            PlayerCommand::Init => literal!("init"),
        }
    }

    fn decode(s: &str) -> Option<Self> {
        match s {
            "init" => Some(PlayerCommand::Init),
            _ => None,
        }
    }

    fn parse_ret_ok(&self, tokens: &[&str]) -> Option<Box<dyn Any + Send>> {
        match self {
            PlayerCommand::Init => {
                CommandInit::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
        }
    }

    fn parse_ret_err(&self, tokens: &[&str]) -> Option<Box<dyn Any + Send>> {
        match self {
            PlayerCommand::Init => {
                CommandInit::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
        }
    }
}
