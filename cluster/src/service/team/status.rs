use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::model::team::Status as TeamStatusKind;

#[derive(Debug)]
pub struct AtomicTeamStatus(AtomicU8);
impl AtomicTeamStatus {
    pub fn set(&self, status: TeamStatusKind) {
        self.0.store(status as u8, Ordering::Relaxed);
    }
    pub fn kind(&self) -> TeamStatusKind {
        TeamStatusKind::from(self.0.load(Ordering::Relaxed))
    }
}

impl Default for AtomicTeamStatus {
    fn default() -> Self {
        AtomicTeamStatus(AtomicU8::new(TeamStatusKind::default() as u8))
    }
}

impl PartialEq<TeamStatusKind> for &AtomicTeamStatus {
    fn eq(&self, other: &TeamStatusKind) -> bool {
        self.kind() == *other
    }
}

impl PartialEq<TeamStatusKind> for AtomicTeamStatus {
    fn eq(&self, other: &TeamStatusKind) -> bool {
        self == other
    }
}