mod team;
mod allies;
mod opponents;
mod team_side;

pub use team::TeamV1;
pub use team_side::TeamSideV1;


use serde::{Deserialize, Serialize};
use crate::schema::Schema;

use allies::AlliesTeamV1;
use opponents::OpponentsTeamV1;


#[derive(Serialize, Clone, Debug)]
pub struct TeamsV1 {
    pub left: TeamV1,
    pub right: TeamV1,
}

impl Schema for TeamsV1 {
    fn verify(&self) -> Result<(), &'static str> {
        if self.left.side == self.right.side {
            return Err("Teams cannot be on the same side")
        }

        if self.left.name == self.right.name {
            return Err("Teams cannot share the same name")
        }

        self.left.verify()?;
        self.right.verify()
    }
}

use serde::de::{self, Deserializer, MapAccess, Visitor};
impl<'de> Deserialize<'de> for TeamsV1 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(TeamsVisitor)
    }
}

struct TeamsVisitor;
impl<'de> Visitor<'de> for TeamsVisitor {
    type Value = TeamsV1;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map with 'left' and 'right' keys")
    }

    fn visit_map<V>(self, mut map: V) -> Result<TeamsV1, V::Error>
    where V: MapAccess<'de>,
    {
        let mut left: Option<AlliesTeamV1> = None;
        let mut right: Option<OpponentsTeamV1> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "left" => {
                    if left.is_some() {
                        return Err(de::Error::duplicate_field("left"));
                    }
                    left = Some(map.next_value()?);
                }
                "right" => {
                    if right.is_some() {
                        return Err(de::Error::duplicate_field("right"));
                    }
                    right = Some(map.next_value()?);
                }
                _ => {
                    return Err(de::Error::unknown_field(&key, &["left", "right"]));
                }
            }
        }

        let left = left.ok_or_else(|| de::Error::missing_field("left"))?;
        let right = right.ok_or_else(|| de::Error::missing_field("right"))?;

        Ok(TeamsV1 {
            left: left.into(),
            right: right.into(),
        })
    }
}
