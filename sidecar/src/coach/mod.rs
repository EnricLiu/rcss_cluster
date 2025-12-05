pub mod command;
mod error;
mod builder;
mod resolver;
mod addon;
mod coach;
mod rich_client;

pub use rich_client::RichClient;
pub use coach::OfflineCoach;
pub use coach::OfflineCoach as Trainer;
pub use error::{Error, Result};
pub use builder::OfflineCoachBuilder as Builder;

pub use resolver::{Sender as CallerSender, WeakSender as CallerWeakSender};
pub use addon::{Addon, CallerAddon};
