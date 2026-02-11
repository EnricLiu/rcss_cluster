mod team;
mod player;
mod policy;
mod position;
mod config;
mod utils;

use super::Schema;

pub use config::Config;

pub use team::{Teams, Team};
pub use player::Player;
pub use policy::Policy;

pub use position::Position;


#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_config() -> Result<(), Box<dyn std::error::Error>> {
        let config = include_str!("../../../config.json");
        let config: super::Config = serde_json::from_str(config)?;
        println!("{:?}",config);

        Ok(())
    }
}
