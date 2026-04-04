use axum::extract::State;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use super::{AppState, Response};

#[derive(Deserialize, Debug)]
pub struct DeleteRequest {
    name: String,
}

#[derive(Serialize, Debug)]
pub struct DeleteResponse {

}

async fn delete(
    State(state): State<AppState>,
    Json(req): Json<DeleteRequest>
) -> Response {
    let res = state.k8s.drop_fleet(&req.name).await;
    match res {
        Ok(_) => Response::success::<()>(None),
        Err(err) => Response::error("TODO", &err.to_string()),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .route("/", axum::routing::delete(delete));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}