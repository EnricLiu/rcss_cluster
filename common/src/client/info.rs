use serde::Serialize;
use chrono::{DateTime, Utc};
use super::{Client, StatusKind};

#[derive(Serialize, Debug, Clone)]
pub struct ClientInfo {
    pub name: String,
    pub status: StatusKind,

    /// The last time the client sent or received a message. precise to second(`TOUCH_INTERVAL` depends).
    pub touched_at: DateTime<Utc>,
}

impl Client {
    pub async fn info(&self) -> ClientInfo {
        ClientInfo {
            name: self.name().to_string(),
            status: self.status(),
            touched_at: self.touched_at(),
        }
    }
}
