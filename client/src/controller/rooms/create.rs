use super::{AppState, Error, Response};
use axum::extract::State;
use axum::{Json, Router, routing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PostRequest {
    pub name: String,
    pub udp_port: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostResponse {
    pub name: String,
}

pub async fn post(State(state): State<AppState>, Json(req): Json<PostRequest>) -> Response {
    let config = state.server.create_room(req.name, req.udp_port).await;

    match config {
        Ok(config) => Response::success(Some(PostResponse { name: config.name })),
        Err(e) => Error::from(e).into(),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
