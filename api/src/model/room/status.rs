use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Copy, PartialEq, Serialize, Deserialize, Clone, Debug)]
#[derive(Default)]
pub enum RoomStatus {
    #[default]
    Idle,
    Waiting,
    Started,
    Finished,
}
impl From<u8> for RoomStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => RoomStatus::Idle,
            1 => RoomStatus::Waiting,
            2 => RoomStatus::Started,
            3 => RoomStatus::Finished,
            _ => panic!("Invalid RoomStatusKind"),
        }
    }
}
