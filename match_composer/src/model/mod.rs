pub mod image;
pub mod player;
pub mod coach;
pub mod team;

use std::fmt::Debug;
use std::path::PathBuf;
use crate::declaration::ImageDeclaration;

pub use team::TeamModel;
pub use image::ImageInfo;
pub use player::{PlayerModel, PlayerBaseModel, PlayerKind};
pub use coach::{CoachBaseModel, CoachKind, CoachModel};

pub trait ProcessModel: Debug + Send + Sync + 'static {
	fn image(&self) -> &ImageDeclaration;
	fn log_dir(&self) -> Option<PathBuf>;
	fn log_file_name(&self) -> String;
	fn process_label(&self) -> String;
}
