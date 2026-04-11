mod status;
mod conf;
mod rcss_conf;
#[cfg(feature = "agones")]
mod agones_conf;
#[cfg(feature = "agones")]
mod agones;

#[cfg(feature = "agones")]
pub use agones_conf::{AgonesConfigInfo, AgonesMcConfigInfo};
#[cfg(feature = "agones")]
pub use agones::{AgonesRuntimeInfo, McLastPollInfo};

pub use conf::ConfigInfo;
pub use status::ServiceStatusInfo;
pub use rcss_conf::RcssConfigInfo;
