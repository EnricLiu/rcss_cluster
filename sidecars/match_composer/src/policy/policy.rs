use std::fmt::Debug;
use crate::image::Image;

pub struct Policy<Cfg: Debug> {
    pub cfg: Cfg,
    pub image: Box<dyn Image>,
}

impl<Cfg: Debug> Debug for Policy<Cfg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bot")
            .field("cfg", &self.cfg)
            .field("image", &format!("{}:{}", self.image.provider(), self.image.model()))
            .finish()
    }
}
