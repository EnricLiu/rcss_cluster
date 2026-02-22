use std::fmt::Debug;
use crate::config::BotConfig;
use crate::image::Image;
use super::BotProcess;

pub struct Bot<'a> {
    pub cfg: BotConfig<'a>,
    pub image: Box<dyn Image>,
}

impl Debug for Bot<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bot")
            .field("cfg", &self.cfg)
            .field("image", &format!("{}:{}", self.image.provider(), self.image.model()))
            .finish()
    }
}

impl<'a> Bot<'a> {
    pub fn new(config: BotConfig<'a>, image: Box<dyn Image>) -> Self {
        Bot {
            cfg: config,
            image,
        }
    }


    pub async fn spawn(&self) -> BotProcess {
        let cmd = self.image.player_cmd(&self.cfg.player());
        BotProcess::spawn(cmd).expect("Failed to spawn bot process")
    }
    
    pub fn unum(&self) -> u8 {
        self.cfg.unum
    }
}
