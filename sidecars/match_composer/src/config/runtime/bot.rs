use std::path::PathBuf;

use common::types::Side;
use crate::player::{PlayerMeta, PolicyMeta};
use super::{ImageQuery, PlayerProcessConfig, ServerConfig};

#[derive(Debug, Clone)]
pub struct BotConfig {
    pub unum: u8,
    pub side: Side,
    pub team: String,
    pub goalie: bool,
    pub image: ImageQuery,
    pub server: ServerConfig,
    pub log_root: Option<PathBuf>,
}

impl From<BotConfig> for PlayerMeta {
    fn from(config: BotConfig) -> Self {
        PlayerMeta {
            unum: config.unum,
            kind: crate::player::PlayerKind::Bot,
            team_name: config.team.clone(),
        }
    }
}

impl BotConfig {
    pub fn player(&self) -> PlayerProcessConfig {
        PlayerProcessConfig {
            host: self.server.host,
            port: self.server.port,
            unum: self.unum,
            goalie: self.goalie,
            team_name: self.team.clone(),
            log_root: self.log_root.clone(),
        }
    }
}
