use std::path::PathBuf;

use common::types::Side;

use super::{ImageConfig, ServerConfig};

#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub unum: u8,
    pub side: Side,
    pub team: String,
    pub goalie: bool,
    pub server: ServerConfig,
    pub grpc: ServerConfig,
    pub image: ImageConfig,
    pub log_root: Option<PathBuf>,
}
