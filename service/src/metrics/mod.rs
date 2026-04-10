mod status;
mod conf;
mod rcss_conf;
#[cfg(feature = "agones")]
mod agones_conf;
mod agones;

#[cfg(feature = "agones")]
pub use agones_conf::{AgonesConfigInfo, AgonesMcConfigInfo};

pub use conf::ConfigInfo;
pub use status::ServiceStatusInfo;
pub use rcss_conf::RcssConfigInfo;
