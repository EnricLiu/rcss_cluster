mod error;
mod proxy;
mod room;

use proxy::ProxyError;
use room::RoomError;

pub use error::Error;

use super::Response;
