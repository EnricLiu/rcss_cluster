use crate::config::{ImageMeta, PlayerProcessConfig};
use std::path::Path;
use tokio::process::Command;

pub trait Image: Send + Sync {
    fn meta(&self) -> &ImageMeta;
    fn provider(&self) -> &str {
        &self.meta().provider
    }
    fn model(&self) -> &str {
        &self.meta().model
    }

    fn path(&self) -> &Path {
        &self.meta().path
    }

    fn cmd(&self) -> Command;
    fn player_cmd(&self, config: &PlayerProcessConfig) -> Command;
}
