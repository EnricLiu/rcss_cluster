use serde::Serialize;
use crate::metrics::RcssConfigInfo;
#[cfg(feature = "agones")]
use crate::metrics::AgonesConfigInfo;

#[derive(Serialize, Debug)]
pub struct ConfigInfo {
    pub base: RcssConfigInfo,
    
    #[cfg(feature = "agones")]
    pub agones: AgonesConfigInfo,
}
