use super::{AppState, Response};
use axum::Router;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRequest {
    pub client_id: Uuid,
}

async fn get(State(state): State<AppState>, Query(request): Query<GetRequest>) -> Response {
    todo!()
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, axum::routing::get(get))
}
