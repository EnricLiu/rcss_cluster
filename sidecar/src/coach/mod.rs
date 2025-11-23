mod coach;
mod signal;
mod error;

use common::client;

pub use coach::OfflineCoach;
pub use coach::OfflineCoach as Trainer;
pub use signal::CoachSignal;
pub use error::{Error, Result};