use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::composer::AgentConnectionInfo;

#[derive(Serialize)]
pub struct AgentConnInfo {
    pub side: String,
    pub unum: u8,
    pub team_name: String,
    pub grpc_host: String,
    pub grpc_port: u16,
}

impl From<&AgentConnectionInfo> for AgentConnInfo {
    fn from(a: &AgentConnectionInfo) -> Self {
        Self {
            side: format!("{:?}", a.side),
            unum: a.unum,
            team_name: a.team_name.clone(),
            grpc_host: a.grpc_host.to_string(),
            grpc_port: a.grpc_port,
        }
    }
}

#[derive(Serialize)]
pub struct StartResponse {
    pub agents: Vec<AgentConnInfo>,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<AgentConnInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: &'static str,
}
