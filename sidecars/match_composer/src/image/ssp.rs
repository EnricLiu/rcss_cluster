use crate::config::{ImageMeta, PlayerProcessConfig};
use crate::image::image::PolicyImage;
use tokio::process::Command;

pub struct SSPImage {
    pub cfg: ImageMeta,
}

impl From<ImageMeta> for SSPImage {
    fn from(cfg: ImageMeta) -> Self {
        SSPImage { cfg }
    }
}

impl PolicyImage for SSPImage {
    fn meta(&self) -> &ImageMeta {
        &self.cfg
    }

    fn cmd(&self) -> Command {
        Command::new(self.meta().path.join("start_player.sh"))
    }
}
