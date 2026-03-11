use crate::player::{PolicyMeta, PolicyPlayer};
use std::fmt::Debug;

pub trait PolicyConfig: Debug + Send + Sync + 'static {
    fn command(&self) -> tokio::process::Command;

    fn meta(&self) -> PolicyMeta;
    
    fn log_dir(&self) -> Option<std::path::PathBuf>;

    fn mkdir(&self) -> std::io::Result<()> {
        if let Some(log_dir) = self.log_dir() {
            std::fs::create_dir_all(log_dir)?;
        }
        Ok(())
    }
}

impl PolicyConfig for Box<dyn PolicyConfig> {
    fn command(&self) -> tokio::process::Command {
        (**self).command()
    }

    fn meta(&self) -> PolicyMeta {
        (**self).meta()
    }

    fn log_dir(&self) -> Option<std::path::PathBuf> {
        (**self).log_dir()
    }
}