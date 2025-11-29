use std::str::FromStr;

use arcstr::{ArcStr, literal, format};
use crate::coach::signal::SignalKind;

pub struct SignalInit {
    pub version: Option<u8>,
}

impl super::Signal for SignalInit {
    type Ok = ();
    type Error = SignalInitError;

    fn kind(&self) -> SignalKind {
        SignalKind::Init
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
pub enum SignalInitError {}

impl FromStr for SignalInitError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <SignalInitError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
