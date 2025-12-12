use axum::{routing, Json, Router};
use axum::extract::State;
use serde::{Deserialize, Serialize};
use super::{AppState, Response, RoomResponse, Error};

#[derive(Debug, Clone, Deserialize)]
pub struct GetRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostResponse(RoomResponse);

pub async fn post(
    State(state): State<AppState>,
    Json(req): Json<GetRequest>,
) -> Response {
    let info = state.server.room_info(&req.name);
    match info {
        Ok(info) => Response::success(info.into()),
        Err(e) => Error::from(e).into(),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::post(post))
}
