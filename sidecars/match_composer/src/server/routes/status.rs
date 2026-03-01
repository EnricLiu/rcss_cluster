use axum::extract::State;
use axum::{Json, Router, routing};

use super::super::AppState;
use super::super::response::StatusResponse;

async fn get(State(state): State<AppState>) -> Json<StatusResponse> {
    state.status().await
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}
