mod client;
mod config;
mod error;
mod kind;
mod signal;
mod status;

pub use client::Client;
pub use client::ClientBuilder as Builder;
pub use client::ClientRxData as RxData;
pub use client::ClientTxData as TxData;
pub use client::ClientTxSignal as TxSignal;
pub use config::ClientConfig as Config;
pub use error::{Error, Result};
pub use kind::ClientKind as Kind;
pub use signal::ClientSignal as Signal;
pub use status::AtomicClientStatus as AtomicStatus;
pub use status::ClientStatusKind as StatusKind;

pub const INIT_MSG_TIMEOUT_MS: u64 = 5000;
pub const BUFFER_SIZE: usize = 1500;
pub const CHANNEL_CAPACITY: usize = 32;
