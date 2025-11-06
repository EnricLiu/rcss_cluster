mod side;
mod status;
mod config;
mod team;
mod error;

pub use team::Team;
pub use error::{Error, Result};
pub use side::TeamSide as Side;
pub use status::TeamStatus as Status;
pub use config::TeamConfig as Config;
