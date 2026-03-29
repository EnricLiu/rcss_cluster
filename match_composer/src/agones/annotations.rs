use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use common::errors::BuilderError;
use crate::declarations::{
    InitStateDeclaration,
    RefereeDeclaration,
    StopEventDeclaration
};


#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Annotations {
    pub team_l: String, // team name
    pub team_r: String,
    pub init: InitStateDeclaration,
    pub referee: RefereeDeclaration,
    pub stopping: StopEventDeclaration,
}

impl Annotations {
    pub fn from_map(mut map: HashMap<String, String>) -> Self {
        let referee = map.get("referee")
            .and_then(|r| serde_json::from_str(r).ok())
            .unwrap_or_default();
        let stopping = map.get("stopping")
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        let init = map.get("init")
            .and_then(|i| serde_json::from_str(i).ok())
            .unwrap_or_default();
        let team_l = map.remove("team.l").unwrap_or("TeamLeft".to_string());
        let team_r = map.remove("team.r").unwrap_or("TeamRight".to_string());
        Annotations { referee, stopping, init, team_l, team_r }
    }
}

impl TryInto<Annotations> for HashMap<String, String> {
    type Error = BuilderError;
    fn try_into(self) -> Result<Annotations, Self::Error> {
        Ok(Annotations::from_map(self))
    }
}

