mod config;
mod conn;
mod error;
mod room;
mod ws;

pub use config::{RoomConfig, WsConfig};
pub use conn::{LazyProxyConnection, ProxyStatus};
pub use error::{Error, Result};
pub use room::{Room, RoomInfo};

use ws::{WsConnection, WsConnector};

pub const HEARTBEAT_DURATION: std::time::Duration = std::time::Duration::from_secs(10);
