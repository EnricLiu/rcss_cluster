use super::{AppState, Error, Response, RoomResponse};
use axum::extract::State;
use axum::{Json, Router, routing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GetRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostResponse(RoomResponse);

pub async fn post(State(state): State<AppState>, Json(req): Json<GetRequest>) -> Response {
    let info = state.server.room_info(&req.name);
    match info {
        Ok(info) => Response::success(info.into()),
        Err(e) => Error::from(e).into(),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
