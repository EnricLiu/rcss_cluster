use tokio::process::Command;
use crate::model::ImageInfo;
use super::PolicyImage;

pub struct HeliosBaseImage {
    pub image: ImageInfo,
}

impl From<ImageInfo> for HeliosBaseImage {
    fn from(image: ImageInfo) -> Self {
        HeliosBaseImage { image }
    }
}

impl PolicyImage for HeliosBaseImage {
    fn image(&self) -> &ImageInfo {
        &self.image
    }

    fn player_cmd(&self) -> Command {
        Command::new(self.image().path.join("start_player.sh"))
    }
    fn coach_cmd(&self) -> Command {
        Command::new(self.image().path.join("start_coach.sh"))
    }
}

