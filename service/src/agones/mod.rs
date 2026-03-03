mod agones;
mod args;
mod config;
mod metadata;

use crate::base::{BaseService, BaseArgs};

pub use args::AgonesArgs;
pub use config::AgonesConfig;
pub use agones::AgonesService;
