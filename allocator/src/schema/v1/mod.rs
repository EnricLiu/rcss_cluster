mod team;
mod player;
mod policy;
mod position;
mod config;
mod utils;
mod agent;

use super::Schema;

pub use config::ConfigV1;

pub use team::{TeamsV1, TeamV1, TeamSideV1};
pub use agent::AgentV1;
pub use player::{CoachV1, PlayerV1};
pub use policy::PolicyV1;

pub use position::Position;


#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_config() -> Result<(), Box<dyn std::error::Error>> {
        let config = include_str!("../../../template.json");
        let config: super::ConfigV1 = serde_json::from_str(config)?;
        println!("{:?}",config);

        Ok(())
    }
}
