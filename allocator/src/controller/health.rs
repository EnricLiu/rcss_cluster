use axum::{http::StatusCode, routing, Json, Router};
use serde_json::{json, Value};
use super::AppState;

pub async fn get() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"status": "healthy"})))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}
