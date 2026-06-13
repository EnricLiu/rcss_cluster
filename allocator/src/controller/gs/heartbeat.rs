use axum::extract::State;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use super::{AppState, Response};

#[derive(Deserialize, Debug)]
pub struct PostRequest {
    name: String,
}

#[derive(Serialize, Debug)]
pub struct PostResponse {}

async fn post(State(state): State<AppState>, Json(req): Json<PostRequest>) -> Response {
    let res = state.k8s.heartbeat_gs(&req.name).await;
    match res {
        Ok(_) => Response::success(PostResponse {}),
        Err(err) => Response::error(err.desc(), &err.to_string()),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new().route("/", axum::routing::post(post));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
