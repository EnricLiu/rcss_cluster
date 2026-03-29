use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use common::errors::BuilderError;
use common::types::Side;

use crate::declarations::PlayerDeclaration;


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerLabel {
    #[serde(flatten)]
    pub player: PlayerDeclaration,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Labels {
    pub left: HashMap<u8, PlayerLabel>, // p.l.1, p.l.2 = {json}
    pub right: HashMap<u8, PlayerLabel>, // p.r.1, p.r.2 = {json}
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
                "l" => Side::LEFT,
                "r" => Side::RIGHT,
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
                Side::LEFT => left.insert(unum, player_label),
                Side::RIGHT => right.insert(unum, player_label),
                _ => unreachable!(),
            };
        }

        todo!()
    }
}

impl TryInto<Labels> for HashMap<String, String> {
    type Error = BuilderError;
    fn try_into(self) -> Result<Labels, Self::Error> {
        Ok(Labels::from_map(self))
    }
}

