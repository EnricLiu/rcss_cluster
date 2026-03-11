use crate::config::{ImageMeta, ImageQuery};
use std::fmt::Debug;
use tokio::process::Command;

pub trait PolicyImage: Send + Sync {
    fn meta(&self) -> &ImageMeta;
    fn query(&self) -> ImageQuery {
        ImageQuery {
            provider: self.meta().provider.to_string(),
            model: self.meta().model.to_string(),
        }
    }

    fn cmd(&self) -> Command;
}

impl Debug for dyn PolicyImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.meta())
    }
}
