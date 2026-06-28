use axum::extract::State;
use axum::{Router, routing, Json};
use axum::response::IntoResponse;
use serde::Serialize;

use service::BaseConfig;

use super::{AppState, Response};

#[derive(Serialize, Debug)]
pub struct GetResponse<'a> {
    #[serde(flatten)]
    base: &'a BaseConfig,
}

async fn get(State(state): State<AppState>) -> Response {
    let resp = GetResponse { base: state.service.base_config() };
    Response::success(resp)
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}
