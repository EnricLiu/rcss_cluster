use std::collections::HashMap;
pub use agones::{ObjectMeta, Sdk};
use serde::{Deserialize, Serialize};
use crate::schema::v1::{ConfigV1, PolicyV1, TeamSideV1};
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerLabel {
    #[serde(flatten)]    policy: PolicyV1,
}

pub struct Labels {
    left: HashMap<u8, PlayerLabel>, // p.l.1, p.r.2 = {json}
    right: HashMap<u8, PlayerLabel>, // p.l.1, p.r.2 = {json}
}

impl Labels {
    pub fn from_map(map: HashMap<String, String>) -> Self {
        let mut left = HashMap::new();
        let mut right = HashMap::new();

        let keys: Vec<_> = map.keys()
            .map(|key| (key, key.split(".").collect::<Vec<_>>())).collect();

        let player_keys = keys.iter().filter_map(|(full, parts)| {
            if parts.len() != 3 { return None }
            if parts[0] != "p" { return None }

            let side = match parts[1] {
                "l" => TeamSideV1::Left,
                "r" => TeamSideV1::Right,
                _ => return None,
            };

            let unum = parts[2].parse::<u8>().ok()?;
            Some((full, side, unum))

        });

        for (key, side, unum) in player_keys {
            let value = map.get(*key).unwrap();
            let player_label = serde_json::from_str::<PlayerLabel>(value)
                .unwrap_or_else(|_| panic!("Failed to parse player label for key {}", key));
            match side {
                TeamSideV1::Left => left.insert(unum, player_label),
                TeamSideV1::Right => right.insert(unum, player_label),
            };
        }

        todo!()
    }
}

pub struct Annotations {
    pub config: String,
}

impl Annotations {
    pub fn from_map(map: HashMap<String, String>) -> Self {
        let config = map.get("config").cloned().unwrap_or_default();
        Annotations { config }
    }
}

impl TryFrom<Annotations> for ConfigV1 {
    type Error = String;

    fn try_from(value: Annotations) -> Result<Self, Self::Error> {
        serde_json::from_str(&value.config)
            .map_err(|e| format!("Failed to parse config annotation: {}", e))
    }
}

#[test]
fn test2() {
    let mut map = HashMap::new();
    let config = include_str!("../docs/template.json");
    map.insert("config".to_string(), config.to_string());

    let annotations = Annotations::from_map(map);
    let config_v1 = ConfigV1::try_from(annotations).unwrap();
    println!("Parsed ConfigV1: {:?}", config_v1);
}