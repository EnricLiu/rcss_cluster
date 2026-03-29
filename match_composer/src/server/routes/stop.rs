use axum::extract::State;
use axum::{Json, Router, routing};

use super::super::{AppState, Error};

async fn post(State(state): State<AppState>) -> Result<Json<()>, Error> {
    state.stop().await?;
    Ok(Json(()))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
