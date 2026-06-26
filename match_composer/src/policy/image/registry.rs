use std::path::Path;

use crate::model::ImageInfo;
use super::{ImageFormat, ImageRole, ManifestImage, PolicyImage};
use super::manifest::ManifestLoadError;

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
                if  let Ok(ty) = ent.file_type() && ty.is_dir() &&
                    let Ok(model) = ent.file_name().into_string() {
                    if model.starts_with('.') {
                        return None;
                    }
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
                    if provider.starts_with('.') {
                        return None;
                    }
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
        match ManifestImage::load(image.clone()) {
            Ok(image) => Some(Box::new(image)),
            Err(ManifestLoadError::Missing) => {
                log::warn!(
                    "Image '{}' does not contain metadata.json, image will not be loaded",
                    image.to_raw()
                );
                
                None
            }
            Err(e) => {
                log::warn!("Failed to load image manifest for '{}': {e}", image.to_raw());
                None
            }
        }
    }

    pub fn role_is_compatible(
        image: &dyn PolicyImage,
        role: ImageRole,
        expected_format: impl Into<ImageFormat>,
    ) -> bool {
        let expected_format = expected_format.into(); 
        
        if image.format() != expected_format {
            log::warn!(
                "Image '{}' declares format {:?}, expected {:?}",
                image.image().to_raw(),
                image.format(),
                expected_format
            );
            return false;
        }

        if !image.supports_role(role) {
            log::warn!(
                "Image '{}' does not support role {:?}",
                image.image().to_raw(),
                role
            );
            return false;
        }

        true
    }
    
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn hub() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("hub")
    }

    #[test]
    fn loads_manifest_for_cls_ssp_with_trainer_role() {
        let registry = ImageRegistry::new(hub());
        let image = registry
            .try_get("CLSFramework", "soccer-simulation-proxy")
            .expect("CLSFramework SSP image should load");

        assert!(image.supports_role(ImageRole::Player));
        assert!(image.supports_role(ImageRole::Coach));
        assert!(image.supports_role(ImageRole::Trainer));
        assert_eq!(image.format(), ImageFormat::Ssp);
    }

    #[test]
    fn helios_manifest_does_not_claim_trainer_support() {
        let registry = ImageRegistry::new(hub());
        let image = registry
            .try_get("HELIOS", "helios-base")
            .expect("HELIOS image should load");

        assert!(image.supports_role(ImageRole::Player));
        assert!(image.supports_role(ImageRole::Coach));
        assert!(!image.supports_role(ImageRole::Trainer));
        assert_eq!(image.format(), ImageFormat::Helios);
    }

    #[test]
    fn compatibility_requires_matching_format_and_role() {
        let registry = ImageRegistry::new(hub());

        let ssp = registry
            .try_get("CLSFramework", "soccer-simulation-proxy")
            .expect("CLSFramework SSP image should load");
        assert!(ImageRegistry::role_is_compatible(
            ssp.as_ref(),
            ImageRole::Trainer,
            ImageFormat::Ssp,
        ));
        assert!(!ImageRegistry::role_is_compatible(
            ssp.as_ref(),
            ImageRole::Trainer,
            ImageFormat::Helios,
        ));

        let helios = registry
            .try_get("HELIOS", "helios-base")
            .expect("HELIOS image should load");
        assert!(ImageRegistry::role_is_compatible(
            helios.as_ref(),
            ImageRole::Player,
            ImageFormat::Helios,
        ));
        assert!(!ImageRegistry::role_is_compatible(
            helios.as_ref(),
            ImageRole::Trainer,
            ImageFormat::Helios,
        ));
    }

    #[test]
    fn all_hub_images_have_registry_metadata() {
        let registry = ImageRegistry::new(hub());
        let providers = registry
            .providers()
            .expect("hub providers should be readable")
            .collect::<Vec<_>>();

        assert!(!providers.iter().any(|provider| provider.starts_with('.')));

        for provider in providers {
            let models = registry
                .models(&provider)
                .expect("hub provider should be readable")
                .collect::<Vec<_>>();

            for model in models {
                let image = registry
                    .try_get(&model.provider, &model.model)
                    .unwrap_or_else(|| panic!("image should load: {}", model.to_raw()));

                assert!(
                    image.supports_role(ImageRole::Player),
                    "image should support player: {}",
                    image.image().to_raw(),
                );
                assert!(
                    image.supports_role(ImageRole::Coach),
                    "image should support coach: {}",
                    image.image().to_raw(),
                );
            }
        }
    }
}
