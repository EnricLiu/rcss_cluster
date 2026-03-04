use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct AllocateRequest {
    pub bot_count: u32,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub env_params: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub client_version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AllocateResponse {
    pub ip: String,
    pub port: u16,
    pub game_server_name: String,
}
