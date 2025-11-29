use std::str::FromStr;

use arcstr::{ArcStr, format};
use common::types::EyeMode;

use super::SignalKind;

pub struct SignalEye {
    pub mode: EyeMode,
}

impl super::Signal for SignalEye {
    type Ok = EyeMode;
    type Error = SignalEyeError;

    fn kind(&self) -> SignalKind {
        SignalKind::Eye
    }
    fn encode(&self) -> ArcStr {
        format!("(eye {})", self.mode.encode())
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        if tokens.len() != 1 { return None }
        tokens[0].parse().ok()
    }

    fn parse_ret_err(tokens: &[&str]) -> Option<Self::Error> {
        if tokens.len() != 1 { return None }
        let tokens = tokens.join(" ");
        if tokens.is_empty() { return None }

        tokens.parse().ok()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SignalEyeError {
    #[error("MODE did not match on or off.")]
    IllegalMode,
    #[error("The MODE argument was omitted.")]
    IllegalCommandForm,
}

impl FromStr for SignalEyeError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <SignalEyeError as FromStr>::Err> {
        match s {
            "illegal_mode" => Ok(Self::IllegalMode),
            "illegal_command_form" => Ok(Self::IllegalCommandForm),
            _ => Err(()),
        }
    }
}