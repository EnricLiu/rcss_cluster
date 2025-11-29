use std::str::FromStr;

use arcstr::{ArcStr, literal};
use common::types;
use crate::coach::signal::SignalKind;

pub struct SignalRecover;

impl super::Signal for SignalRecover {
    type Ok = ();
    type Error = SignalRecoverError;

    fn kind(&self) -> SignalKind {
        SignalKind::Recover
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
pub enum SignalRecoverError {}

impl FromStr for SignalRecoverError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <SignalRecoverError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
