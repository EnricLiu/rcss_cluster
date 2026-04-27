use std::path::Path;

use super::image::ImageRegistry;
use crate::model::coach::{CoachBaseModel, CoachModel};
use crate::model::player::{PlayerBaseModel, PlayerModel};
use crate::policy::{CoachPolicy, PlayerPolicy, Policy};

pub struct PolicyRegistry {
    pub images: ImageRegistry,
}

impl PolicyRegistry {
    pub fn new(image_registry_path: impl AsRef<Path>) -> Self {
        PolicyRegistry {
            images: ImageRegistry::new(image_registry_path),
        }
    }
    
    pub fn fetch(&self, player: PlayerModel) -> Result<Box<dyn Policy<Model = PlayerBaseModel>>, PlayerModel> {
        let image = self.images.try_get(&player.image.provider(), &player.image.model());
        let image = match image {
            Some(image) => image,
            None => return Err(player),
        };
        
        let ret = match player {
            PlayerModel::Helios(helios) => {
                Box::new(PlayerPolicy::new(helios, image)) as Box<dyn Policy<Model = PlayerBaseModel>>
            },
            PlayerModel::Ssp(ssp) => {
                Box::new(PlayerPolicy::new(ssp, image)) as Box<dyn Policy<Model = PlayerBaseModel>>
            },
        };

        Ok(ret)
    }

    pub fn fetch_coach(&self, coach: CoachModel) -> Result<Box<dyn Policy<Model = CoachBaseModel>>, CoachModel> {
        let image = self.images.try_get(&coach.image.provider(), &coach.image.model());
        let image = match image {
            Some(image) => image,
            None => return Err(coach),
        };

        let ret = match coach {
            CoachModel::Helios(helios) => {
                Box::new(CoachPolicy::new(helios, image)) as Box<dyn Policy<Model = CoachBaseModel>>
            },
            CoachModel::Ssp(ssp) => {
                Box::new(CoachPolicy::new(ssp, image)) as Box<dyn Policy<Model = CoachBaseModel>>
            },
        };

        Ok(ret)
    }
}
