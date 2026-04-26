use std::path::PathBuf;
use tokio::process::Command;
use crate::model::player::{PlayerBaseModel, SspPlayerModel};
use super::{PlayerPolicy, Policy};

impl Policy for PlayerPolicy<SspPlayerModel> {
    fn command(&self) -> Command {
        let mut cmd = self.image.cmd();
        let config = &self.player;
        cmd
            .arg("-h").arg(config.server.ip().to_string())
            .arg("-p").arg(config.server.port().to_string())
            .arg("-t").arg(&config.team)
            .arg("-u").arg(config.unum.to_string())
            .arg("--g-ip").arg(config.grpc.ip().to_string())
            .arg("--g-port").arg(config.grpc.port().to_string());

        if let Some(image_log_root) = &config.log_root {
            cmd.arg("--debug")
                .arg("--log-dir")
                .arg(image_log_root);
        }

        if config.goalie {
            cmd.arg("-g");
        }
        
        cmd
    }

    fn parse_ready_fn(&self) -> fn(&str) -> bool  {
        |line: &str| line.contains("init ok.")
    }

    fn info(&self) -> &PlayerBaseModel {
        self.player.as_ref()
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.player.log_root.clone()
    }
}
