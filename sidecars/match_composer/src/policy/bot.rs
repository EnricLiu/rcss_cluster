use std::path::Path;
use crate::config::BotConfig;
use super::{Image, ImageRegistry};

pub struct Bot {
    pub cfg: BotConfig,
    pub image: Image,
    
}

impl Bot {
    pub fn new(config: BotConfig, image: Image) -> Self {
        Bot {
            cfg: config,
            image,
        }
    }
}

pub struct BotRegistry {
    pub images: ImageRegistry
}

impl BotRegistry {
    pub fn new(local: impl AsRef<Path>) -> Self {
        BotRegistry {
            images: ImageRegistry::new(local)
        }
    }
    
    pub fn build_bot(&self, config: BotConfig) -> Option<Bot> {
        let image = self.images.try_get(&config.image.provider, &config.image.model)?;
        let ret = Bot::new(config, image);
        Some(ret)
    }
}