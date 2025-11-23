use uuid::Uuid;
use axum::extract::{Query, State};
use axum::Router;
use serde::Deserialize;
use super::{AppState, Response};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRequest {
    pub client_id: Uuid,
}

async fn get(
    State(state): State<AppState>,
    Query(request): Query<GetRequest>
) -> Response {
    todo!()
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, axum::routing::get(get))
}