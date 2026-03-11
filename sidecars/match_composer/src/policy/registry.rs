use crate::config::{AgentConfig, BotConfig, PlayerConfig};
use crate::image::ImageRegistry;
use crate::policy::agent::PolicyAgentConfig;
use crate::policy::bot::PolicyBotConfig;
use std::path::Path;
use crate::policy::PolicyConfig;

pub struct PolicyRegistry {
    pub images: ImageRegistry,
}

impl PolicyRegistry {
    pub fn new(image_registry_path: impl AsRef<Path>) -> Self {
        PolicyRegistry {
            images: ImageRegistry::new(image_registry_path),
        }
    }

    pub fn fetch_bot(&self, bot: BotConfig) -> Result<PolicyBotConfig, BotConfig> {
        let image = self.images.try_get(&bot.image.provider, &bot.image.model);
        
        if let Some(image) = image {
            Ok(PolicyBotConfig::new(bot, image))
        } else {
            Err(bot)
        }
    }

    pub fn fetch_agent(&self, agent: AgentConfig) -> Result<PolicyAgentConfig, AgentConfig> {
        let image = self.images.try_get(&agent.image.provider, &agent.image.model);
        if let Some(image) = image {
            Ok(PolicyAgentConfig::new(agent, image))
        } else {
            Err(agent)
        }
    }
    
    pub fn fetch(&self, config: PlayerConfig) -> Result<Box<dyn PolicyConfig>, PlayerConfig> {
        match config { 
            PlayerConfig::Bot(bot) =>
                self.fetch_bot(bot)
                    .map(|cfg| Box::new(cfg) as Box<dyn PolicyConfig>)
                    .map_err(|bot| PlayerConfig::Bot(bot)),
            
            PlayerConfig::Agent(agent) =>
                self.fetch_agent(agent)
                    .map(|cfg| Box::new(cfg) as Box<dyn PolicyConfig>)
                    .map_err(|agent| PlayerConfig::Agent(agent)),
        }
    }
}
