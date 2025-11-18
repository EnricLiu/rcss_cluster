use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::model::room::Status as RoomStatusKind;

#[derive(Debug)]
pub struct AtomicRoomStatus(AtomicU8);
impl AtomicRoomStatus {
    pub fn set(&self, status: RoomStatusKind) {
        self.0.store(status as u8, Ordering::Relaxed);
    }
    pub fn kind(&self) -> RoomStatusKind {
        RoomStatusKind::from(self.0.load(Ordering::Relaxed))
    }
}

impl Default for AtomicRoomStatus {
    fn default() -> Self {
        AtomicRoomStatus(AtomicU8::new(RoomStatusKind::default() as u8))
    }
}

impl PartialEq<RoomStatusKind> for &AtomicRoomStatus {
    fn eq(&self, other: &RoomStatusKind) -> bool {
        self.kind() == *other
    }
}

impl PartialEq<RoomStatusKind> for AtomicRoomStatus {
    fn eq(&self, other: &RoomStatusKind) -> bool {
        self == other
    }
}