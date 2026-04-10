use serde::Serialize;
use chrono::{DateTime, Utc};
use crate::ServerStatus;

#[derive(Serialize, Debug, Clone)]
pub struct ServiceStatusInfo {
    pub status: ServerStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestep: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_ms: Option<i64>,
}

impl crate::Service {
    pub async fn status_info(&self) -> ServiceStatusInfo {
        let status = self.status_now();
        let timestep = self.time_now().await;
        let started_at = self.started_at().await;
        let uptime_ms = started_at
            .map(|started_at| (Utc::now() - started_at).num_milliseconds());

        ServiceStatusInfo {
            status,
            timestep,
            started_at,
            uptime_ms,
        }
    }
}