use std::path::PathBuf;
use super::{PolicyConfig};
use crate::config::AgentConfig;
use crate::image::PolicyImage;
use crate::player::{PlayerKind, PlayerMeta, PolicyMeta};
use tokio::process::Command;

#[derive(Debug)]
pub struct PolicyAgentConfig {
    pub cfg: AgentConfig,
    pub image: Box<dyn PolicyImage>,
}

impl PolicyConfig for PolicyAgentConfig {
    fn command(&self) -> Command {
        let mut cmd = self.image.cmd();
        cmd
            .arg("-h").arg(self.cfg.server.host.to_string())
            .arg("-p").arg(self.cfg.server.port.to_string())
            .arg("-t").arg(&self.cfg.team)
            .arg("-u").arg(self.cfg.unum.to_string())
            .arg("--g-ip").arg(self.cfg.grpc.host.to_string())
            .arg("--g-port").arg(self.cfg.grpc.port.to_string());

        if let Some(image_log_root) = &self.cfg.log_root {
            cmd.arg("--debug")
                .arg("--log-dir")
                .arg(image_log_root);
        }
        
        cmd
    }

    fn meta(&self) -> PolicyMeta {
        let player = PlayerMeta {
            unum: self.cfg.unum,
            kind: PlayerKind::Agent {
                grpc: self.cfg.grpc.clone(),
            },
            team_name: self.cfg.team.clone(),
        };

        let image = self.image.meta().clone();

        PolicyMeta {
            player,
            image,
        }
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.cfg.log_root.clone()
    }
}

impl PolicyAgentConfig {
    pub fn new(config: AgentConfig, image: Box<dyn PolicyImage>) -> Self {
        PolicyAgentConfig {
            cfg: config,
            image,
        }
    }
}
