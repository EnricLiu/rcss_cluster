use axum::extract::State;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::{AppState, Response};

#[derive(Deserialize, Debug)]
pub struct PostRequest {
    name: String,
    version: u8,
    conf: Value,
}

#[derive(Serialize, Debug)]
pub struct PostResponse {

}

async fn post(
    State(state): State<AppState>,
    Json(req): Json<PostRequest>
) -> Response {
    let res = state.k8s.create_fleet(req.name, req.conf, req.version).await;
    match res {
        Ok(_) => Response::success::<()>(None),
        Err(err) => Response::error("TODO", &err.to_string()),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .route("/", axum::routing::post(post));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}