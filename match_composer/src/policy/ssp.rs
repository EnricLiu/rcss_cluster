use std::path::PathBuf;
use tokio::process::Command;
use crate::model::player::{PlayerBaseModel, SspPlayerModel};
use super::{PlayerPolicy, Policy};

impl Policy for PlayerPolicy<SspPlayerModel> {
    fn command(&self) -> Command {
        let mut cmd = self.image.cmd();
        let config = &self.player;
        cmd
            .arg("-h").arg(config.server.host.to_string())
            .arg("-p").arg(config.server.port.to_string())
            .arg("-t").arg(&config.team)
            .arg("-u").arg(config.unum.to_string())
            .arg("--g-ip").arg(config.grpc.host.to_string())
            .arg("--g-port").arg(config.grpc.port.to_string());

        if let Some(image_log_root) = &config.log_root {
            cmd.arg("--debug")
                .arg("--log-dir")
                .arg(image_log_root);
        }
        
        cmd
    }

    fn info(&self) -> &PlayerBaseModel {
        self.player.as_ref()
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.player.log_root.clone()
    }
}
