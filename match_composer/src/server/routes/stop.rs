use axum::extract::State;
use axum::{Router, routing};
use serde::{Deserialize, Serialize};
use common::axum::response::Response;
use super::super::{AppState, Error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostRequest {

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostResponse {

}

async fn post(State(state): State<AppState>) -> Response {
    let _res = match state.stop().await {
        Err(e) => return e.into(),
        Ok(res) => res,
    };
    Response::success(PostResponse {})
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
