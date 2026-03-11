use std::path::PathBuf;

use common::types::Side;
use crate::player::{PlayerKind, PlayerMeta};
use super::{ImageQuery, ServerConfig};

#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub unum: u8,
    pub side: Side,
    pub team: String,
    pub goalie: bool,
    pub server: ServerConfig,
    pub grpc: ServerConfig,
    pub image: ImageQuery,
    pub log_root: Option<PathBuf>,
}

impl From<AgentConfig> for PlayerMeta {
    fn from(config: AgentConfig) -> Self {
        PlayerMeta {
            unum: config.unum,
            kind: PlayerKind::Agent {
                grpc: config.grpc.clone()
            },
            team_name: config.team.clone(),
        }
    }
}
