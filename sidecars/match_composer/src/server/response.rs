use crate::team::TeamMeta;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct StartResponse {

}

#[derive(Serialize)]
pub struct StatusResponse {
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_l: Option<TeamMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_r: Option<TeamMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: &'static str,
}
