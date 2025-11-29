use std::str::FromStr;

use arcstr::{ArcStr, format};
use common::types;

use super::SignalKind;

pub struct SignalChangeMode {
    pub play_mode: types::PlayMode,
}

impl super::Signal for SignalChangeMode {
    type Ok = ();
    type Error = SignalChangeModeError;

    fn kind(&self) -> SignalKind {
        SignalKind::ChangeMode
    }
    
    fn encode(&self) -> ArcStr {
        format!("(change_mode {})", self.play_mode.encode())
    }

    fn parse_ret_err(tokens: &[&str]) -> Option<Self::Error> {
        if tokens.len() != 1 { return None }
        let tokens = tokens.join(" ");
        if tokens.is_empty() { return None }

        tokens.parse().ok()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SignalChangeModeError {
    #[error("The specified mode was not valid.")]
    IllegalMode,
    #[error("The PLAY_MODE argument was omitted")]
    IllegalCommandForm,
}

impl FromStr for SignalChangeModeError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <SignalChangeModeError as FromStr>::Err> {
        match s {
            "illegal_mode" => Ok(Self::IllegalMode),
            "illegal_command_form" => Ok(Self::IllegalCommandForm),
            _ => Err(()),
        }
    }
}
