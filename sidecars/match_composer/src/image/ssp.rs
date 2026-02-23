use tokio::process::Command;
use crate::config::{ImageConfig, PlayerProcessConfig};
use crate::image::image::Image;

pub struct SSPImage {
    pub cfg: ImageConfig,
}

impl From<ImageConfig> for SSPImage {
    fn from(cfg: ImageConfig) -> Self {
        SSPImage {
            cfg,
        }
    }
}

impl Image for SSPImage {
    fn cfg(&self) -> &ImageConfig {
        &self.cfg
    }

    fn cmd(&self) -> Command {
        Command::new(self.path().join("start_player.sh"))
    }

    fn player_cmd(&self, config: &PlayerProcessConfig) -> Command {
        todo!("Not implemented for SSPImage")
    }
}
