use axum::Router;
use serde::Serialize;

use super::{AppState, Response};


#[derive(Serialize, Debug)]
pub struct PostResponse<'a> {
    version: &'a str,
}

async fn get() -> Response {
    let template = crate::k8s::fleet_template_version();
    Response::success(Some(PostResponse { version: template }))
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .route("/", axum::routing::get(get));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
