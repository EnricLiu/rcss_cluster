use crate::config::PlayerSchema;
use crate::schema::v1::TeamV1;

#[derive(Clone, Debug)]
pub struct TeamSchema {
    pub name: String,
    pub players: Vec<PlayerSchema>,
}

impl From<TeamV1> for TeamSchema {
    fn from(team: TeamV1) -> Self {
        let players = team
            .players
            .into_iter()
            .map(PlayerSchema::from)
            .collect();

        TeamSchema {
            name: team.name,
            players,
        }
    }
}
