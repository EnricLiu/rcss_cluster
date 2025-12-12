mod room;
mod proxy;
mod error;

use room::RoomError;
use proxy::ProxyError;

pub use error::Error;

use super::Response;
