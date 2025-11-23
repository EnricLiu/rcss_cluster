mod config;
mod kind;
mod error;
mod status;
mod client;
mod signal;

pub use error::{Error, Result};
pub use config::ClientConfig as Config;
pub use kind::ClientKind as Kind;
pub use status::AtomicClientStatus as AtomicStatus;
pub use status::ClientStatusKind as StatusKind;
pub use signal::ClientSignal as Signal;
pub use client::Client;


pub const INIT_MSG_TIMEOUT_MS: u64 = 5000;
pub const BUFFER_SIZE: usize = 1500;
pub const CHANNEL_CAPACITY: usize = 32;