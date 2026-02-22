use tokio::process::Command;
use crate::config::{ImageConfig, PlayerProcessConfig};
use crate::image::image::Image;

pub struct HeliosBaseImage {
    pub cfg: ImageConfig,
}

impl From<ImageConfig> for HeliosBaseImage {
    fn from(cfg: ImageConfig) -> Self {
        HeliosBaseImage {
            cfg,
        }
    }
}

impl Image for HeliosBaseImage {
    fn cfg(&self) -> &ImageConfig {
        &self.cfg
    }

    fn player_cmd(&self, config: &PlayerProcessConfig) -> Command {
        let mut cmd = Command::new(self.path().join("start_player.sh"));
        cmd.arg("-h")
            .arg(config.host.to_string())
            .arg("-p")
            .arg(config.port.to_string())
            .arg("-t")
            .arg(config.team_name)
            .arg("-u")
            .arg(config.unum.to_string())
            .arg("--log-dir")
            .arg(config.log_path);

        if config.goalie {
            cmd.arg("-g");
        }
        cmd
    }
}
