use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug)]
#[serde(rename_all="snake_case")]
pub enum TeamSide {
    Left,
    Right
}

impl TeamSide {
    pub fn allies() -> TeamSide {
        TeamSide::Left
    }
    pub fn opponents() -> TeamSide {
        TeamSide::Right
    }
}
