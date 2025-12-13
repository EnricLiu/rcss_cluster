use super::{AppState, Response};
use axum::{Router, routing};
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GetResponse {
    pub status: String,
    pub timestamp: String,
}

pub async fn get() -> Response {
    let payload = GetResponse {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };

    Response::success(Some(payload))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}
