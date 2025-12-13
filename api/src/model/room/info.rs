use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::model::{room, team};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomInfo {
    pub room_id: Uuid,
    pub name: String,

    pub team_l: Option<team::Info>,
    pub team_r: Option<team::Info>,

    pub status: room::Status,
}
