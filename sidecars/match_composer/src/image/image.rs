use std::path::Path;
use tokio::process::Command;
use crate::config::{ImageConfig, PlayerProcessConfig};

pub trait Image {
    fn cfg(&self) -> &ImageConfig;
    fn provider(&self) -> &str {
        &self.cfg().provider
    }
    fn model(&self) -> &str {
        &self.cfg().model
    }

    fn path(&self) -> &Path {
        &self.cfg().path
    }
    
    fn cmd(&self) -> Command;
    fn player_cmd(&self, config: &PlayerProcessConfig) -> Command;
}