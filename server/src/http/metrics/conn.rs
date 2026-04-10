use std::collections::HashMap;
use axum::extract::State;
use axum::{Router, routing};
use serde::Serialize;
use uuid::Uuid;
use common::client::Info as ClientInfo;
use super::{AppState, Response};

#[derive(Serialize, Debug)]
pub struct GetResponse {
    status: HashMap<Uuid, ClientInfo>,
}

async fn get(State(state): State<AppState>) -> Response {
    let status = state.conn_info().await;
    Response::success(Some(GetResponse { status }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::get(get))
}
