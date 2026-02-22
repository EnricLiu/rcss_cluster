use std::path::Path;
use common::types::Side;
use crate::config::{ImageQuery, PlayerProcessConfig};
use crate::config::server::ServerConfig;

#[derive(Debug, Clone)]
pub struct BotConfig<'a> {
    pub unum: u8,
    pub side: Side,
    pub team: &'a str,
    pub image: &'a ImageQuery,
    pub server: &'a ServerConfig,
    
    pub log_path: &'a Path,
}

impl<'a> BotConfig<'a> {
    pub fn player(&self) -> PlayerProcessConfig<'a> {
        PlayerProcessConfig {
            host: self.server.host,
            port: self.server.port,
            unum: self.unum,
            goalie: false,
            team_name: self.team,
            log_path: self.log_path,
        }
    }
}
