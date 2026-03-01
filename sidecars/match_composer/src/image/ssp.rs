use crate::config::{ImageMeta, PlayerProcessConfig};
use crate::image::image::Image;
use tokio::process::Command;

pub struct SSPImage {
    pub cfg: ImageMeta,
}

impl From<ImageMeta> for SSPImage {
    fn from(cfg: ImageMeta) -> Self {
        SSPImage { cfg }
    }
}

impl Image for SSPImage {
    fn meta(&self) -> &ImageMeta {
        &self.cfg
    }

    fn cmd(&self) -> Command {
        Command::new(self.path().join("start_player.sh"))
    }

    fn player_cmd(&self, _config: &PlayerProcessConfig) -> Command {
        unimplemented!("SSPImage uses cmd() with manual args, player_cmd() is not supported")
    }
}
