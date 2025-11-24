use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientStatusKind {
    Idle = 0,
    WaitingRedirection = 1,
    Connected = 2,
    Disconnected = 3,
    Died = 4, // 🫥
}

impl ClientStatusKind {
    pub fn is_connected(&self) -> bool {
        match self {
            ClientStatusKind::Connected => true,
            _ => false,
        }
    }
    
    pub fn is_running(&self) -> bool {
        match self {
            ClientStatusKind::Disconnected | ClientStatusKind::Died => false,
            _ => true,
        }
    }
}

impl Default for ClientStatusKind {
    fn default() -> Self {
        ClientStatusKind::Disconnected
    }
}
impl From<u8> for ClientStatusKind {
    fn from(value: u8) -> Self {
        match value {
            0 => ClientStatusKind::Idle,
            1 => ClientStatusKind::WaitingRedirection,
            2 => ClientStatusKind::Connected,
            3 => ClientStatusKind::Disconnected,
            4 => ClientStatusKind::Died,
            _ => panic!("Invalid ClientStateEnum"),
        }
    }
}

#[derive(Debug)]
pub struct AtomicClientStatus(AtomicU8);
impl AtomicClientStatus {
    pub fn set(&self, status: ClientStatusKind) {
        self.0.store(status as u8, Ordering::Relaxed);
    }
    pub fn kind(&self) -> ClientStatusKind {
        ClientStatusKind::from(self.0.load(Ordering::Relaxed))
    }
}

impl Default for AtomicClientStatus {
    fn default() -> Self {
        AtomicClientStatus(AtomicU8::new(ClientStatusKind::default() as u8))
    }
}

impl PartialEq<ClientStatusKind> for &AtomicClientStatus {
    fn eq(&self, other: &ClientStatusKind) -> bool {
        self.kind() == *other
    }
}

impl PartialEq<ClientStatusKind> for AtomicClientStatus {
    fn eq(&self, other: &ClientStatusKind) -> bool {
        self == other
    }
}