use serde::{Deserialize, Serialize};
use crate::schema::v1::{CoachV1, Schema, PlayerV1, TeamV1};
use super::TeamSideV1;
use super::team::verify_team;


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OpponentsTeamV1 {
    pub name: String,
    #[serde(default="TeamSideV1::opponents")]
    pub side: TeamSideV1,
    pub players: Vec<PlayerV1>,
    #[serde(default)]
    pub coach: Option<CoachV1>,
}

impl Schema for OpponentsTeamV1 {
    fn verify(&self) -> Result<(), &'static str> {
        verify_team(&self.name, &self.players)?;
        if let Some(coach) = &self.coach {
            coach.verify()?;
        }
        Ok(())
    }
}

impl From<OpponentsTeamV1> for TeamV1 {
    fn from(val: OpponentsTeamV1) -> Self {
        TeamV1 {
            name: val.name,
            side: val.side,
            players: val.players,
            coach: val.coach,
        }
    }
}
