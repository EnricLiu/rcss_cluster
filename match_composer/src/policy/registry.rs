use std::path::Path;

use super::image::ImageRegistry;
use crate::model::player::PlayerModel;
use crate::policy::{PlayerPolicy, Policy};

pub struct PolicyRegistry {
    pub images: ImageRegistry,
}

impl PolicyRegistry {
    pub fn new(image_registry_path: impl AsRef<Path>) -> Self {
        PolicyRegistry {
            images: ImageRegistry::new(image_registry_path),
        }
    }
    
    pub fn fetch(&self, player: PlayerModel) -> Result<Box<dyn Policy>, PlayerModel> {
        let image = self.images.try_get(&player.image.provider(), &player.image.model());
        let image = match image {
            Some(image) => image,
            None => return Err(player),
        };
        
        let ret = match player {
            PlayerModel::Helios(helios) => {
                Box::new(PlayerPolicy::new(helios, image)) as Box<dyn Policy>
            },
            PlayerModel::Ssp(ssp) => {
                Box::new(PlayerPolicy::new(ssp, image)) as Box<dyn Policy>
            },
        };
        
        Ok(ret)
    }
}
