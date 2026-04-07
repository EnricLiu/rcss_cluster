use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use common::errors::{BuilderError, BuilderResult};
use common::types::Side;

use crate::declaration::{PlayerDeclaration, Unum};
use super::label_serdes;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerLabel {
    #[serde(flatten)]
    pub player: PlayerDeclaration,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Labels {
    pub left: HashMap<Unum, PlayerLabel>, // p.l.1, p.l.2 = str
    pub right: HashMap<Unum, PlayerLabel>, // p.r.1, p.r.2 = str
}

impl Labels {
    pub fn from_map(mut map: HashMap<String, String>) -> BuilderResult<Self> {
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
            let unum = Unum::try_from(unum)?;

            let player_label = label_serdes::des::<PlayerLabel>(&(unum, value.clone()))
                .map_err(|e| -> BuilderError { e.into() })?;


            match side {
                Side::LEFT => left.insert(unum, player_label),
                Side::RIGHT => right.insert(unum, player_label),
                _ => unreachable!(),
            };
        }

        Ok(
            Self {
                left,
                right,
            }
        )
    }

    pub fn try_into_map(self) -> BuilderResult<HashMap<String, String>> {
        let mut map = HashMap::with_capacity(self.left.len() + self.right.len());
        for (unum, label) in self.left {
            let encoded = label_serdes::ser(&label)?;
            map.insert(format!("p.l.{}", unum), encoded);
        }
        for (unum, label) in self.right {
            let encoded = label_serdes::ser(&label)?;
            map.insert(format!("p.r.{}", unum), encoded);
        }
        Ok(map)
    }
}

impl TryInto<Labels> for HashMap<String, String> {
    type Error = BuilderError;
    fn try_into(self) -> Result<Labels, Self::Error> {
        Labels::from_map(self)
    }
}
