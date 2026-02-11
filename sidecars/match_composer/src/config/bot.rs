use crate::config::ImageConfig;
use crate::schema::v1::{Player, Policy};

pub struct BotConfig {
    pub unum: u8,
    pub goalie: bool,
    pub image: ImageConfig,
    
}

impl TryFrom<Player> for BotConfig {
    type Error = ();

    fn try_from(player: Player) -> Result<Self, Self::Error> {
        let unum = player.unum;
        let goalie = player.goalie;
        let image = ImageConfig::try_from(player.policy)?;
        
        Ok(BotConfig {
            unum,
            goalie,
            image,
        })
    }
}