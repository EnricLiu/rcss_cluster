use crate::policy::bot::BotPolicy;
use crate::config::{AgentConfig, BotConfig};
use crate::image::{ImageRegistry, SSPImage};
use crate::policy::agent::AgentPolicy;

pub struct PolicyRegistry {
    pub images: ImageRegistry,
}

impl PolicyRegistry {
    pub fn new(image_registry_path: &str) -> Self {
        PolicyRegistry {
            images: ImageRegistry::new(image_registry_path),
        }
    }

    pub fn fetch_bot<'a>(&self, bot: BotConfig<'a>) -> Option<BotPolicy<'a>> {
        let image = self.images.try_get(bot.image.clone())?;
        let bot = BotPolicy::new(bot, image);
        Some(bot)
    }

    pub fn fetch_agent<'a>(&self, agent: AgentConfig<'a>) -> Option<AgentPolicy<'a>> {
        let image = SSPImage::from(agent.image.clone());
        let image = Box::new(image);
        let agent = AgentPolicy::new(agent, image);
        Some(agent)
    }
}