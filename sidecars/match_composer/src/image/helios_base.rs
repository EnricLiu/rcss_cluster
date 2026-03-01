use crate::config::{ImageMeta, PlayerProcessConfig};
use crate::image::image::Image;
use tokio::process::Command;

pub struct HeliosBaseImage {
    pub cfg: ImageMeta,
}

impl From<ImageMeta> for HeliosBaseImage {
    fn from(cfg: ImageMeta) -> Self {
        HeliosBaseImage { cfg }
    }
}

impl Image for HeliosBaseImage {
    fn meta(&self) -> &ImageMeta {
        &self.cfg
    }

    fn cmd(&self) -> Command {
        Command::new(self.path().join("start_player.sh"))
    }

    fn player_cmd(&self, config: &PlayerProcessConfig) -> Command {
        let mut cmd = self.cmd();
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
}
