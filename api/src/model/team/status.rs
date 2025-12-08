use serde::{Serialize, Deserialize};

#[repr(u8)]
#[derive(Copy, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum TeamStatus {
    Idle,
    Playing,
}
impl Default for TeamStatus {
    fn default() -> Self {
        TeamStatus::Idle
    }
}
impl From<u8> for TeamStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => TeamStatus::Idle,
            1 => TeamStatus::Playing,
            _ => panic!("Invalid RoomStatusKind"),
        }
    }
}
