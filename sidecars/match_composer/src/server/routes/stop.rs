use axum::extract::State;
use axum::{Json, Router, routing};

use super::super::{AppState, Error};
use super::super::response::MessageResponse;

async fn post(State(state): State<AppState>) -> Result<Json<MessageResponse>, Error> {
    state.stop().await?;
    Ok(Json(MessageResponse { message: "Composer stopped." }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
