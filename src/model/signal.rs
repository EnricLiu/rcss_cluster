use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Payload {
    Control(),
    Raw(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Signal {
    id:         Uuid,
    resp_id:    Option<Uuid>,
    room_id:    Uuid,
    payload:    Payload,
}

pub struct PayloadPlayer {
    raw: String,

}