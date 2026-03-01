use std::net::Ipv4Addr;
use std::path::PathBuf;

use common::types::Side;

use super::{AgentConfig, BotConfig, ImageConfig, ServerConfig};
use crate::config::schema::{PlayerPolicySchema, PlayerSchema};

#[derive(Clone, Debug)]
pub struct PlayerProcessConfig {
    pub host: Ipv4Addr,
    pub port: u16,
    pub unum: u8,
    pub goalie: bool,
    pub team_name: String,
    pub log_root: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub enum PlayerConfig {
    Bot(BotConfig),
    Agent(AgentConfig),
}

impl PlayerConfig {
    pub fn unum(&self) -> u8 {
        match self {
            PlayerConfig::Bot(bot) => bot.unum,
            PlayerConfig::Agent(agent) => agent.unum,
        }
    }

    pub fn from_schema(
        player: &PlayerSchema,
        team: &str,
        side: Side,
        server: &ServerConfig,
        log_root: &Option<PathBuf>
        ,
    ) -> Result<Self, String> {
        let image_value = match &player.policy {
            PlayerPolicySchema::Bot { image } => image.as_str(),
            PlayerPolicySchema::Agent { image, .. } => image.as_str(),
        };
        let image = ImageConfig::try_from(image_value)
            .map_err(|_| "Invalid policy image format".to_string())?;

        match &player.policy {
            PlayerPolicySchema::Bot { .. } => Ok(PlayerConfig::Bot(BotConfig {
                side,
                team: team.to_string(),
                server: server.clone(),
                image,
                unum: player.unum,
                goalie: player.goalie,
                log_root: log_root.clone(),
            })),
            PlayerPolicySchema::Agent {
                grpc_host,
                grpc_port,
                ..
            } => Ok(PlayerConfig::Agent(AgentConfig {
                side,
                team: team.to_string(),
                server: server.clone(),
                grpc: ServerConfig {
                    host: *grpc_host,
                    port: *grpc_port,
                },
                image,
                unum: player.unum,
                goalie: player.goalie,
                log_root: log_root.clone(),
            })),
        }
    }
}
