use crate::config::BotConfig;
use crate::image::{Image, ImageProcess};
use super::Policy;

pub type BotPolicy = Policy<BotConfig>;

impl BotPolicy {
    pub fn new(config: BotConfig, image: Box<dyn Image>) -> Self {
        BotPolicy {
            cfg: config,
            image,
        }
    }

    pub async fn spawn(&self) -> ImageProcess {
        let cmd = self.image.player_cmd(&self.cfg.player());

        let stdout_log_path = self.cfg.log_root.as_ref().map(|p| {
            p.join(format!("{}_{:02}_stdout.log", &self.cfg.team, self.cfg.unum))
        });

        println!("Spawning agent with command: {:?}", cmd);

        ImageProcess::spawn(cmd, stdout_log_path.map(|p| p.into_boxed_path()))
            .expect("Failed to spawn bot process")
    }

    pub fn unum(&self) -> u8 {
        self.cfg.unum
    }
}
