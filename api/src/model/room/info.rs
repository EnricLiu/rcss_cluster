use serde::{Serialize, Deserialize};

use uuid::Uuid;

use crate::model::{team, room};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomInfo {
    pub room_id: Uuid,
    pub name: String,

    pub team_l: Option<team::Info>,
    pub team_r: Option<team::Info>,

    pub status: room::Status,
}
