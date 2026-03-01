use std::path::PathBuf;

use common::types::Side;

use super::{ServerConfig, TeamConfig};
use crate::config::schema::TeamSchema;
use crate::schema::v1::ConfigV1;

#[derive(Clone, Debug)]
pub struct MatchComposerConfig {
    pub server: ServerConfig,
    pub log_root: Option<PathBuf>,
    pub allies: TeamConfig,
    pub opponents: TeamConfig,
}

impl MatchComposerConfig {
    pub fn from_schema(cfg: ConfigV1, log_root: Option<PathBuf>) -> Result<Self, String> {
        let server = ServerConfig {
            host: cfg.host,
            port: cfg.port,
        };

        let allies_log_root = log_root.as_ref().map(|p| p.join("allies"));
        let opponents_log_root =  log_root.as_ref().map(|p| p.join("opponents"));

        let allies = TeamConfig::from_schema(
            TeamSchema::from(cfg.teams.allies),
            Side::LEFT,
            &server,
            &allies_log_root,
        )?;

        let opponents = TeamConfig::from_schema(
            TeamSchema::from(cfg.teams.opponents),
            Side::RIGHT,
            &server,
            &opponents_log_root,
        )?;

        Ok(MatchComposerConfig {
            server,
            log_root,
            allies,
            opponents,
        })
    }
}
