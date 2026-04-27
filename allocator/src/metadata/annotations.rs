use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use common::errors::BuilderError;
use crate::declaration::{
    CoachDeclaration,
    InitStateDeclaration,
    RefereeDeclaration,
    StopEventDeclaration
};


#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Annotations {
    pub team_l: String, // team name
    pub team_r: String,
    pub coach_l: Option<CoachDeclaration>,
    pub coach_r: Option<CoachDeclaration>,
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
        let coach_l = map.get("team.coach.l")
            .and_then(|c| serde_json::from_str(c).ok());
        let coach_r = map.get("team.coach.r")
            .and_then(|c| serde_json::from_str(c).ok());
        Annotations { referee, stopping, init, team_l, team_r, coach_l, coach_r }
    }
    pub fn into_map(self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("team.l".to_string(), self.team_l);
        map.insert("team.r".to_string(), self.team_r);
        if let Some(coach_l) = self.coach_l && let Ok(coach_str) = serde_json::to_string(&coach_l) {
            map.insert("team.coach.l".to_string(), coach_str);
        }
        if let Some(coach_r) = self.coach_r && let Ok(coach_str) = serde_json::to_string(&coach_r) {
            map.insert("team.coach.r".to_string(), coach_str);
        }
        if let Ok(referee_str) = serde_json::to_string(&self.referee) {
            map.insert("referee".to_string(), referee_str);
        }
        if let Ok(stopping_str) = serde_json::to_string(&self.stopping) {
            map.insert("stopping".to_string(), stopping_str);
        }
        if let Ok(init_str) = serde_json::to_string(&self.init) {
            map.insert("init".to_string(), init_str);
        }
        map
    }
}

impl TryInto<Annotations> for HashMap<String, String> {
    type Error = BuilderError;
    fn try_into(self) -> Result<Annotations, Self::Error> {
        Ok(Annotations::from_map(self))
    }
}

