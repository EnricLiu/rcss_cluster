use std::str::FromStr;

use arcstr::{ArcStr, format};
use common::types::EarMode;

use super::SignalKind;

pub struct SignalEar{
    pub mode: EarMode,
}

impl super::Signal for SignalEar {
    type Ok = EarMode;
    type Error = SignalEarError;

    fn kind(&self) -> SignalKind {
        SignalKind::Ear
    }
    fn encode(&self) -> ArcStr {
        format!("(ear {})", self.mode.encode())
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        if tokens.len() != 1 { return None }
        tokens[0].parse().ok()
    }

    fn parse_ret_err(tokens: &[&str]) -> Option<Self::Error> {
        if tokens.len() != 1 { return None }
        tokens[0].parse().ok()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SignalEarError {
    #[error("MODE did not match on or off.")]
    IllegalMode,
    #[error("The MODE argument was omitted.")]
    IllegalCommandForm,
}

impl FromStr for SignalEarError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <SignalEarError as FromStr>::Err> {
        match s {
            "illegal_mode" => Ok(Self::IllegalMode),
            "illegal_command_form" => Ok(Self::IllegalCommandForm),
            _ => Err(()),
        }
    }
}