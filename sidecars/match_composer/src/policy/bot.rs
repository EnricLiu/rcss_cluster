use std::path::PathBuf;
use crate::config::BotConfig;
use crate::image::{PolicyImage};
use crate::player::{PlayerMeta, PolicyMeta};
use super::{PolicyConfig};

#[derive(Debug)]
pub struct PolicyBotConfig {
    pub config: BotConfig,
    pub image: Box<dyn PolicyImage>,
}

impl PolicyConfig for PolicyBotConfig {
    fn command(&self) -> tokio::process::Command {
        let config = &self.config.player();
        
        let mut cmd = self.image.cmd();
        cmd.arg("-h")
            .arg(config.host.to_string())
            .arg("-p")
            .arg(config.port.to_string())
            .arg("-t")
            .arg(&config.team_name)
            .arg("-u")
            .arg(config.unum.to_string());

        if let Some(log_root) = &config.log_root {
            cmd.arg("--debug")
                .arg("--log-dir")
                .arg(log_root);
        }

        if config.goalie {
            cmd.arg("-g");
        }
        
        cmd
    }

    fn meta(&self) -> PolicyMeta {
        
        let player = PlayerMeta {
            unum: self.config.unum,
            kind: crate::player::PlayerKind::Bot,
            team_name: self.config.team.clone(),
        };

        let image = self.image.meta().clone();
        
        PolicyMeta {
            player,
            image,
        }
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.config.log_root.clone()
    }
}

impl PolicyBotConfig {
    pub fn new(config: BotConfig, image: Box<dyn PolicyImage>) -> Self {
        PolicyBotConfig {
            config,
            image,
        }
    }
}
