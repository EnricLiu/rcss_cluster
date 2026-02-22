use crate::policy::bot::Bot;
use crate::config::{BotConfig};
use crate::image::{ImageRegistry};

pub struct BotRegistry {
    pub images: ImageRegistry,
}

impl BotRegistry {
    pub fn new(image_registry_path: &str) -> Self {
        BotRegistry {
            images: ImageRegistry::new(image_registry_path),
        }
    }

    pub fn fetch_bot<'a>(&self, bot: BotConfig<'a>) -> Option<Bot<'a>> {
        let image = self.images.try_get(bot.image.clone())?;
        let bot = Bot::new(bot, image);
        Some(bot)
    }
}