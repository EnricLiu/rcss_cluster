use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

#[repr(u8)]
#[derive(Copy, PartialEq, Clone, Debug)]
pub enum RoomStatusKind {
    Idle,
    Waiting,
    Started,
    Finished,
}
impl Default for RoomStatusKind {
    fn default() -> Self {
        RoomStatusKind::Idle
    }
}
impl From<u8> for RoomStatusKind {
    fn from(value: u8) -> Self {
        match value {
            0 => RoomStatusKind::Idle,
            1 => RoomStatusKind::Waiting,
            2 => RoomStatusKind::Started,
            3 => RoomStatusKind::Finished,
            _ => panic!("Invalid RoomStatusKind"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RoomStatus(Arc<AtomicU8>);
impl RoomStatus {
    pub fn set(&self, status: RoomStatusKind) {
        self.0.store(status as u8, Ordering::Relaxed);
    }
    pub fn kind(&self) -> RoomStatusKind {
        RoomStatusKind::from(self.0.load(Ordering::Relaxed))
    }
}

impl Default for RoomStatus {
    fn default() -> Self {
        RoomStatus(Arc::new(AtomicU8::new(RoomStatusKind::default() as u8)))
    }
}

impl PartialEq<RoomStatusKind> for &RoomStatus {
    fn eq(&self, other: &RoomStatusKind) -> bool {
        self.kind() == *other
    }
}

impl PartialEq<RoomStatusKind> for RoomStatus {
    fn eq(&self, other: &RoomStatusKind) -> bool {
        self == other
    }
}