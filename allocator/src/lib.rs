#[cfg(feature = "agones")]
mod k8s;
#[cfg(feature = "agones")]
mod controller;
mod args;

pub mod schema;
pub mod metadata;
pub mod declaration;
mod utils;

pub use args::AllocateMode;
pub use metadata::MetaData;

#[cfg(feature = "agones")]
pub use controller::AppState;
