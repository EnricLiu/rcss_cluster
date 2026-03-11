mod client;
mod coached;
mod process;
mod test;
mod trainer;
mod player;
mod error;

pub mod addon {
    pub use super::client::{Addon, CallerAddon, RawAddon};
}

pub mod resolver {
    pub use crate::client::{CallResolver, CallSender, WeakCallSender};
}

pub use client::CommandCaller;
pub use coached::{CoachedProcess, CoachedProcessSpawner};
pub use process::Config as ProcessConfig;
pub use common::process::ProcessStatus;
pub use error::{Result, Error};

pub use player::{Player};

pub const RCSS_PROCESS_NAME: &str = "rcssserver";
