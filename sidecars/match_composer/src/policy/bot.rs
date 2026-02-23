use std::path::PathBuf;

use crate::image::{Image, ImageProcess};
use crate::config::BotConfig;
use super::Policy;

pub type BotPolicy<'a> = Policy<BotConfig<'a>>;


impl<'a> BotPolicy<'a> {
    pub fn new(config: BotConfig<'a>, image: Box<dyn Image>) -> Self {
        BotPolicy {
            cfg: config,
            image,
        }
    }

    pub async fn spawn(&self) -> ImageProcess {
        let cmd = self.image.player_cmd(&self.cfg.player());
        ImageProcess::spawn(cmd, Some(PathBuf::from("./logs/test.log").into_boxed_path()))
            .expect("Failed to spawn bot process")
    }

    pub fn unum(&self) -> u8 {
        self.cfg.unum
    }
}
