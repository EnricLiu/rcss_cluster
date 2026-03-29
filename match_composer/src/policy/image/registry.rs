use std::path::Path;

use crate::model::ImageInfo;
use super::{HeliosBaseImage, PolicyImage, SSPImage};

pub struct ImageRegistry {
    pub local: Box<Path>,
}

impl ImageRegistry {
    pub fn new(local: impl AsRef<Path>) -> ImageRegistry {
        ImageRegistry {
            local: local.as_ref().into(),
        }
    }

    pub fn models(&self, provider: &str) -> Option<impl Iterator<Item=ImageInfo>> {
        let dir = match self.local.join(provider).read_dir() {
            Ok(dir) => dir,
            Err(_) => return None,
        };

        let ret = dir.filter_map(|entry| {
            entry.ok().and_then(|ent| {
                if  let Ok(ty) = ent.file_type() && ty.is_file() &&
                    let Ok(model) = ent.file_name().into_string() {
                    return Some(ImageInfo {
                        provider: provider.to_string(),
                        model,
                        path: ent.path().into()
                    })
                }
                None
            })
        });

        Some(ret)
    }

    pub fn providers(&self) -> Option<impl Iterator<Item=String>> {
        let dir = match self.local.read_dir() {
            Ok(dir) => dir,
            Err(_) => return None,
        };

        let ret = dir.filter_map(|entry| {
            entry.ok().and_then(|ent| {
                if  let Ok(ty) = ent.file_type() && ty.is_dir() &&
                    let Ok(provider) = ent.file_name().into_string() {
                    return Some(provider)
                }
                None
            })
        });

        Some(ret)
    }
    
    pub fn try_get(&self, provider: &str, model: &str) -> Option<Box<dyn PolicyImage>> {
        let dir = self.local.join(provider).join(model);
        let meta = dir.is_dir().then_some(
            ImageInfo {
                provider: provider.to_string(),
                model: model.to_string(),
                path: dir.into(),
            }
        )?;
        
        Self::load_image(meta)
    }
    
    fn load_image(image: ImageInfo) -> Option<Box<dyn PolicyImage>> {
        if &image.model == "SoccerSimulationProxy" {
            return Some(Box::new(SSPImage::from(image)));
        }
        
        Some(Box::new(HeliosBaseImage::from(image)))
    }
    
}
