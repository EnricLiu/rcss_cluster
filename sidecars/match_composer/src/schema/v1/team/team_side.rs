use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug)]
#[serde(rename_all="snake_case")]
pub enum TeamSideV1 {
    Left,
    Right
}

impl TeamSideV1 {
    pub fn allies() -> TeamSideV1 {
        TeamSideV1::Left
    }
    pub fn opponents() -> TeamSideV1 {
        TeamSideV1::Right
    }
}
