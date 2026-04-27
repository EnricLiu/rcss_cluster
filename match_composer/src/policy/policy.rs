use std::fmt::Debug;
use crate::model::ProcessModel;
use super::image::PolicyImage;

pub trait Policy: Debug + Send + Sync + 'static {
    type Model: ProcessModel;

    fn command(&self) -> tokio::process::Command;
    fn parse_ready_fn(&self) -> fn(&str) -> bool;

    fn info(&self) -> &Self::Model;

    fn log_dir(&self) -> Option<std::path::PathBuf>;

    fn mkdir(&self) -> std::io::Result<()> {
        if let Some(log_dir) = self.log_dir() {
            std::fs::create_dir_all(log_dir)?;
        }
        Ok(())
    }
}

impl<M: ProcessModel> Policy for Box<dyn Policy<Model = M>> {
    type Model = M;

    fn command(&self) -> tokio::process::Command {
        (**self).command()
    }
    fn parse_ready_fn(&self) -> fn(&str) -> bool {
        (**self).parse_ready_fn()
    }

    fn info(&self) -> &Self::Model {
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


#[derive(Debug)]
pub struct CoachPolicy<C> {
    pub coach: C,
    pub image: Box<dyn PolicyImage>,
}

impl<C> CoachPolicy<C> {
    pub fn new(coach: C, image: Box<dyn PolicyImage>) -> Self {
        Self {
            coach,
            image,
        }
    }
}

impl<P> PlayerPolicy<P> {
    pub fn new(player: P, image: Box<dyn PolicyImage>) -> Self {
        Self {
            player,
            image,
        }
    }
}

