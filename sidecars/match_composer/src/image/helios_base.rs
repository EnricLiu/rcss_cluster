use crate::config::ImageMeta;
use crate::image::image::PolicyImage;
use tokio::process::Command;

pub struct HeliosBaseImage {
    pub cfg: ImageMeta,
}

impl From<ImageMeta> for HeliosBaseImage {
    fn from(cfg: ImageMeta) -> Self {
        HeliosBaseImage { cfg }
    }
}

impl PolicyImage for HeliosBaseImage {
    fn meta(&self) -> &ImageMeta {
        &self.cfg
    }

    fn cmd(&self) -> Command {
        Command::new(self.meta().path.join("start_player.sh"))
    }
}

