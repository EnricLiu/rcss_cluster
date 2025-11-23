use uuid::Uuid;
use axum::extract::State;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use super::{AppState, Response};

use crate::service::room;

pub type PostRequest = room::Config;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    pub room_id: Uuid,
}

pub async fn post(
    State(s): State<AppState>,
    Json(req): Json<PostRequest>
) -> Response {
    let room_id = s.cluster.create_room(req);
    Response::success(Some(PostResponse { room_id }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, axum::routing::post(post))
}