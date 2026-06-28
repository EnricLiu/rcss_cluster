use std::fmt::Debug;
use tokio::process::Command;
use crate::model::{CoachModel, ImageInfo, PlayerModel, TrainerModel};
use crate::declaration::ImageDeclaration;
use super::{ImageFormat, ImageRole};

pub trait PolicyImage: Send + Sync {
    fn image(&self) -> &ImageInfo;
    fn declare(&self) -> Option<ImageDeclaration> {
        ImageDeclaration::try_from(self.image().to_raw()).ok()
    }

    fn format(&self) -> ImageFormat;
    fn command(&self, role: ImageRole) -> Option<Command>;

    fn ready_stdout_contains(&self, _role: ImageRole) -> Option<&str> {
        Some("init ok.")
    }

    fn parse_ready_fn(&self, role: ImageRole) -> Box<dyn Fn(&str) -> bool + Send + 'static> {
        let needle = self.ready_stdout_contains(role).unwrap_or("init ok.").to_string();
        Box::new(move |line: &str| line.contains(&needle))
    }

    fn supports_role(&self, role: ImageRole) -> bool {
        self.command(role).is_some()
    }

    fn player_cmd(&self) -> Command {
        self.command(ImageRole::Player)
            .expect("player command should be validated by PolicyRegistry")
    }

    fn coach_cmd(&self) -> Command {
        self.command(ImageRole::Coach)
            .expect("coach command should be validated by PolicyRegistry")
    }

    fn trainer_cmd(&self) -> Command {
        self.command(ImageRole::Trainer)
            .expect("trainer command should be validated by PolicyRegistry")
    }
}

impl Debug for dyn PolicyImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.image())
    }
}

impl From<&PlayerModel> for ImageFormat {
    fn from(m: &PlayerModel) -> Self {
        match m {
            PlayerModel::Helios(_) => ImageFormat::Helios,
            PlayerModel::Ssp(_) => ImageFormat::Ssp,
        }
    }
}
impl From<&CoachModel> for ImageFormat {
    fn from(m: &CoachModel) -> Self {
        match m {
            CoachModel::Helios(_) => ImageFormat::Helios,
            CoachModel::Ssp(_) => ImageFormat::Ssp,
        }
    }
}
impl From<&TrainerModel> for ImageFormat {
    fn from(m: &TrainerModel) -> Self {
        match m {
            TrainerModel::Helios(_) => ImageFormat::Helios,
            TrainerModel::Ssp(_) => ImageFormat::Ssp,
        }
    }
}
