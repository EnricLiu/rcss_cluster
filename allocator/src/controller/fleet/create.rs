use axum::extract::State;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::{AppState, Response};

#[derive(Deserialize, Debug)]
pub struct PostRequest {
    version: u8,
    conf: Value,
    timeout_ms: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct PostResponse {

}

async fn post(
    State(state): State<AppState>,
    Json(req): Json<PostRequest>
) -> Response {
    let timeout = req.timeout_ms.map(std::time::Duration::from_millis);

    let res = state.k8s.get_or_create_fleet(req.conf, req.version, timeout).await;
    match res {
        Ok(_) => Response::success(PostResponse {}),
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
