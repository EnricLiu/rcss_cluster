mod room;
mod conn;
mod ws;
mod error;
mod config;

pub use room::{LazyRoom, Room, RoomInfo, RoomStatus};
pub use conn::{LazyProxyConnection, ProxyStatus};
pub use config::{RoomConfig, WsConfig};
pub use error::{Result, Error};

use ws::{WsConnection, WsConnector};

pub const HEARTBEAT_DURATION: std::time::Duration = std::time::Duration::from_secs(10);
