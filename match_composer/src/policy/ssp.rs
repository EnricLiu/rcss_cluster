use std::path::PathBuf;
use tokio::process::Command;
use crate::model::coach::{CoachBaseModel, SspCoachModel};
use crate::model::player::{PlayerBaseModel, SspPlayerModel};
use super::{CoachPolicy, PlayerPolicy, Policy};

impl Policy for PlayerPolicy<SspPlayerModel> {
    type Model = PlayerBaseModel;

    fn command(&self) -> Command {
        let mut cmd = self.image.player_cmd();
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

impl Policy for CoachPolicy<SspCoachModel> {
    type Model = CoachBaseModel;

    fn command(&self) -> Command {
        let mut cmd = self.image.coach_cmd();
        let config = &self.coach;
        cmd
            .arg("-h").arg(config.server.ip().to_string())
            .arg("-p").arg(config.server.port().to_string())
            .arg("-t").arg(&config.team)
            .arg("--g-ip").arg(config.grpc.ip().to_string())
            .arg("--g-port").arg(config.grpc.port().to_string());

        if let Some(log_root) = &config.log_root {
            cmd.arg("--debug")
                .arg("--log-dir")
                .arg(log_root);
        }

        cmd
    }

    fn parse_ready_fn(&self) -> fn(&str) -> bool {
        |line: &str| line.contains("init ok.")
    }

    fn info(&self) -> &CoachBaseModel {
        self.coach.as_ref()
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.coach.log_root.clone()
    }
}
