use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Copy, PartialEq, Serialize, Deserialize, Clone, Debug)]
#[derive(Default)]
pub enum TeamStatus {
    #[default]
    Idle,
    Playing,
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
