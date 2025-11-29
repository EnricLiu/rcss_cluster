use std::str::FromStr;

use arcstr::{ArcStr, literal};
use crate::coach::signal::SignalKind;

pub struct SignalLook;
impl super::Signal for SignalLook {
    type Ok = ();
    type Error = SignalLookError;

    fn kind(&self) -> SignalKind {
        SignalKind::Look
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
pub enum SignalLookError {}

impl FromStr for SignalLookError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <SignalLookError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
