use axum::extract::State;
use axum::{Json, Router, routing};
use serde::{Deserialize, Serialize};
use common::axum::response::Response;
use crate::info::GameInfo;
use super::super::AppState;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse {
    pub in_match: bool,
    #[serde(flatten)]
    pub info: Option<GameInfo>,
}

async fn get(State(state): State<AppState>) -> Response {
    let game = state.game.read().await;
    let (in_match, info) = match game.as_ref() {
        Some(game) => (true, Some(game.info())),
        None => (false, None),
    };
    Response::success(GetResponse {
        in_match, info
    })
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}
