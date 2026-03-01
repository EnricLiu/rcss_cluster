use crate::config::{AgentConfig, BotConfig};
use crate::image::ImageRegistry;
use crate::policy::agent::AgentPolicy;
use crate::policy::bot::BotPolicy;
use std::path::Path;

pub struct PolicyRegistry {
    pub images: ImageRegistry,
}

impl PolicyRegistry {
    pub fn new(image_registry_path: impl AsRef<Path>) -> Self {
        PolicyRegistry {
            images: ImageRegistry::new(image_registry_path),
        }
    }

    pub fn fetch_bot(&self, bot: BotConfig) -> Option<BotPolicy> {
        let image = self.images.try_get(bot.image.clone())?;
        let bot = BotPolicy::new(bot, image);
        Some(bot)
    }

    pub fn fetch_agent(&self, agent: AgentConfig) -> Option<AgentPolicy> {
        let image = self.images.try_get(agent.image.clone())?;
        let agent = AgentPolicy::new(agent, image);
        Some(agent)
    }
}
