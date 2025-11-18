use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct TeamConfig {
    pub name: String,
    pub max_players: usize,
}

impl Default for TeamConfig {
    fn default() -> Self {
        Self {
            name: "BTW".to_string(),
            max_players: 11,
        }
    }
}

