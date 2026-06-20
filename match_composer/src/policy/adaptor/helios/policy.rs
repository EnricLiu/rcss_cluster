use std::path::PathBuf;
use tokio::process::Command;

use crate::model::player::{HeliosPlayerModel, PlayerBaseModel};
use crate::model::coach::{CoachBaseModel, HeliosCoachModel};
use crate::model::trainer::{HeliosTrainerModel, TrainerBaseModel};
use crate::policy::{PlayerPolicy, Policy, CoachPolicy, ReadyMatcher, TrainerPolicy};
use crate::policy::image::ImageRole;


impl Policy for PlayerPolicy<HeliosPlayerModel> {
    type Model = PlayerBaseModel;

    fn command(&self) -> Command {
        let config = &self.player;

        let mut cmd = self.image.player_cmd();
        cmd.arg("-h")
            .arg(config.server.ip().to_string())
            .arg("-p")
            .arg(config.server.port().to_string())
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

    fn parse_ready_fn(&self) -> ReadyMatcher {
        self.image.parse_ready_fn(ImageRole::Player)
    }

    fn info(&self) -> &PlayerBaseModel {
        &self.player
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.player.log_root.clone()
    }
}


impl Policy for CoachPolicy<HeliosCoachModel> {
    type Model = CoachBaseModel;

    fn command(&self) -> Command {
        let mut cmd = self.image.coach_cmd();
        let config = &self.coach;
        cmd.arg("-h")
            .arg(config.server.ip().to_string())
            .arg("-p")
            .arg(config.server.port().to_string())
            .arg("-t")
            .arg(&config.team);

        if let Some(log_root) = &config.log_root {
            cmd.arg("--debug")
                .arg("--log-dir")
                .arg(log_root);
        }

        cmd
    }

    fn parse_ready_fn(&self) -> ReadyMatcher {
        self.image.parse_ready_fn(ImageRole::Coach)
    }

    fn info(&self) -> &CoachBaseModel {
        self.coach.as_ref()
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.coach.log_root.clone()
    }
}

impl Policy for TrainerPolicy<HeliosTrainerModel> {
    type Model = TrainerBaseModel;

    fn command(&self) -> Command {
        let mut cmd = self.image.trainer_cmd();
        let config = &self.trainer;
        cmd.arg("-h")
            .arg(config.server.ip().to_string())
            .arg("-p")
            .arg(config.server.port().to_string())
            .arg("-t")
            .arg(&config.team);

        if let Some(log_root) = &config.log_root {
            cmd.arg("--debug")
                .arg("--log-dir")
                .arg(log_root);
        }

        cmd
    }

    fn parse_ready_fn(&self) -> ReadyMatcher {
        self.image.parse_ready_fn(ImageRole::Trainer)
    }

    fn info(&self) -> &TrainerBaseModel {
        self.trainer.as_ref()
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.trainer.log_root.clone()
    }
}
