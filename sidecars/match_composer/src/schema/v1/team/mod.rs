mod team;
mod allies;
mod opponents;
mod team_side;

pub use team::Team;
pub use team_side::TeamSide;


use serde::{Deserialize, Serialize};
use crate::schema::Schema;

use allies::AlliesTeam;
use opponents::OpponentsTeam;


#[derive(Serialize, Clone, Debug)]
pub struct Teams {
    pub allies:     Team,
    pub opponents:  Team,
}

impl Schema for Teams {
    fn verify(&self) -> Result<(), &'static str> {
        if self.allies.side == self.opponents.side {
            return Err("Teams cannot be on the same side")
        }

        if self.allies.name == self.opponents.name {
            return Err("Teams cannot share the same name")
        }

        self.allies.verify()?;
        self.opponents.verify()
    }
}

use serde::de::{self, Deserializer, MapAccess, Visitor};
impl<'de> Deserialize<'de> for Teams {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(TeamsVisitor)
    }
}

struct TeamsVisitor;
impl<'de> Visitor<'de> for TeamsVisitor {
    type Value = Teams;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map with 'allies' and 'opponents' keys")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Teams, V::Error>
    where V: MapAccess<'de>,
    {
        let mut allies: Option<AlliesTeam> = None;
        let mut opponents: Option<OpponentsTeam> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "allies" => {
                    if allies.is_some() {
                        return Err(de::Error::duplicate_field("allies"));
                    }
                    allies = Some(map.next_value()?);
                }
                "opponents" => {
                    if opponents.is_some() {
                        return Err(de::Error::duplicate_field("opponents"));
                    }
                    opponents = Some(map.next_value()?);
                }
                _ => {
                    return Err(de::Error::unknown_field(&key, &["allies", "opponents"]));
                }
            }
        }

        let allies = allies.ok_or_else(|| de::Error::missing_field("allies"))?;
        let opponents = opponents.ok_or_else(|| de::Error::missing_field("opponents"))?;

        Ok(Teams {
            allies: allies.into(),
            opponents: opponents.into(),
        })
    }
}
