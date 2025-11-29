use std::str::FromStr;

use arcstr::{ArcStr, format};
use common::types;
use crate::coach::signal::SignalKind;

pub struct SignalMove {
    pub todo: (),
}

impl super::Signal for SignalMove {
    type Ok = ();
    type Error = SignalMoveError;

    fn kind(&self) -> SignalKind {
        SignalKind::Move
    }

    fn encode(&self) -> ArcStr {
        todo!()
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        tokens.is_empty().then_some(())
    }

    fn parse_ret_err(tokens: &[&str]) -> Option<Self::Error> {
        todo!("really complex too")
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SignalMoveError {
    #[error("The specified mode was not valid.")]
    IllegalMode,
    #[error("The PLAY_MODE argument was omitted")]
    IllegalCommandForm,
}

impl FromStr for SignalMoveError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <SignalMoveError as FromStr>::Err> {
        match s {
            "illegal_mode" => Ok(Self::IllegalMode),
            "illegal_command_form" => Ok(Self::IllegalCommandForm),
            _ => Err(()),
        }
    }
}
