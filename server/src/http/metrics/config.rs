use axum::extract::State;
use axum::{Router, routing};
use serde::Serialize;


use super::{AppState, Response};

#[derive(Serialize, Debug)]
pub struct GetResponse {
    
}

async fn get(State(state): State<AppState>) -> Response {
    todo!()
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}
