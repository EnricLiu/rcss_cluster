use std::path::Path;

use crate::config::{ImageConfig, ImageMeta};
use crate::image::{HeliosBaseImage, Image, SSPImage};

pub struct ImageRegistry {
    pub local: Box<Path>,
}

impl ImageRegistry {
    pub fn new(local: impl AsRef<Path>) -> ImageRegistry {
        ImageRegistry {
            local: local.as_ref().into(),
        }
    }

    pub fn models(&self, provider: &str) -> Option<impl Iterator<Item=ImageMeta>> {
        let dir = match self.local.join(provider).read_dir() {
            Ok(dir) => dir,
            Err(_) => return None,
        };

        let ret = dir.filter_map(|entry| {
            entry.ok().and_then(|ent| {
                if  let Ok(ty) = ent.file_type() && ty.is_file() &&
                    let Ok(model) = ent.file_name().into_string() {
                    return Some(ImageMeta {
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
    
    pub fn try_get(&self, image: ImageConfig) -> Option<Box<dyn Image>> {
        let dir = self.local.join(&image.provider).join(&image.model);
        let meta = dir.is_dir().then_some(
            ImageMeta {
                provider: image.provider,
                model: image.model,
                path: dir.into(),
            }
        )?;
        
        Self::load_image(meta)
    }
    
    fn load_image(meta: ImageMeta) -> Option<Box<dyn Image>> {
        if &meta.model == "SoccerSimulationProxy" {
            return Some(Box::new(SSPImage::from(meta)));
        }
        
        Some(Box::new(HeliosBaseImage::from(meta)))
    }
    
}
