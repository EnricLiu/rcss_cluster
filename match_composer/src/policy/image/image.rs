use std::fmt::Debug;
use tokio::process::Command;
use crate::model::ImageInfo;
use crate::declarations::ImageDeclaration;

pub trait PolicyImage: Send + Sync {
    fn image(&self) -> &ImageInfo;
    fn declare(&self) -> Option<ImageDeclaration> {
        ImageDeclaration::try_from(self.image().to_raw()).ok()
    }

    fn cmd(&self) -> Command;
}

impl Debug for dyn PolicyImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.image())
    }
}
