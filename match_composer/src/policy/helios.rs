use std::path::PathBuf;
use tokio::process::Command;
use crate::model::player::{HeliosPlayerModel, PlayerBaseModel};
use crate::policy::policy::PlayerPolicy;
use super::Policy;

impl Policy for PlayerPolicy<HeliosPlayerModel> {
    fn command(&self) -> Command {
        let config = &self.player;

        let mut cmd = self.image.cmd();
        cmd.arg("-h")
            .arg(config.server.host.to_string())
            .arg("-p")
            .arg(config.server.port.to_string())
            .arg("-t")
            .arg(&config.team)
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

    fn info(&self) -> &PlayerBaseModel {
        &self.player
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.player.log_root.clone()
    }
}
