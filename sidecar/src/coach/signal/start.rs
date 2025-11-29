use std::str::FromStr;

use arcstr::{ArcStr, literal};
use crate::coach::signal::SignalKind;

pub struct SignalStart;
impl super::Signal for SignalStart {
    type Ok = ();
    type Error = SignalStartError;

    fn kind(&self) -> SignalKind {
        SignalKind::Start
    }

    fn encode(&self) -> ArcStr {
        literal!("(start)")
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        tokens.is_empty().then_some(())
    }

    // never error
}

#[derive(thiserror::Error, Debug)]
pub enum SignalStartError {}

impl FromStr for SignalStartError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <SignalStartError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
