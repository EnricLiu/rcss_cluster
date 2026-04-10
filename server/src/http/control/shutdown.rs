use super::{AppState, Response};
use axum::extract::State;
use axum::{Json, Router, routing};
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostRequest {
    #[serde(default)]
    force: bool
}
async fn post(State(state): State<AppState>, req: Option<Json<PostRequest>>) -> Response {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    
    if !req.force && state.service.status_now().is_running() {
        return Response::error(
            "Shutdown Failed",
            "Service is still running. Use force=true to force shutdown."
        );
    }
    
    let res = state.service.shutdown().await;
    match res {
        Ok(_) => Response::success::<()>(None),
        Err(e) => Response::error("Restart Failed", &e.to_string()),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
