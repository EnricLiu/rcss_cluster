use std::fmt::Debug;
use crate::model::player::PlayerBaseModel;
use super::image::PolicyImage;

pub trait Policy: Debug + Send + Sync + 'static {
    
    fn command(&self) -> tokio::process::Command;
    fn parse_ready_fn(&self) -> fn(&str) -> bool;

    fn info(&self) -> &PlayerBaseModel;
    
    fn log_dir(&self) -> Option<std::path::PathBuf>;

    fn mkdir(&self) -> std::io::Result<()> {
        if let Some(log_dir) = self.log_dir() {
            std::fs::create_dir_all(log_dir)?;
        }
        Ok(())
    }
}

impl Policy for Box<dyn Policy> {
    fn command(&self) -> tokio::process::Command {
        (**self).command()
    }
    fn parse_ready_fn(&self) -> fn(&str) -> bool {
        (**self).parse_ready_fn()
    }

    fn info(&self) -> &PlayerBaseModel {
        (**self).info()
    }

    fn log_dir(&self) -> Option<std::path::PathBuf> {
        (**self).log_dir()
    }
}


#[derive(Debug)]
pub struct PlayerPolicy<P> {
    pub player: P,
    pub image: Box<dyn PolicyImage>,
}

impl<P> PlayerPolicy<P> {
    pub fn new(player: P, image: Box<dyn PolicyImage>) -> Self {
        Self {
            player,
            image,
        }
    }
}

