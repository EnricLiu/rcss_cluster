mod drop;
mod allocate;
mod heartbeat;

use axum::Router;
use super::{Error, Result, AppState, Response};


pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(allocate::route("/allocate"))
        .merge(drop::route("/drop"))
        .merge(heartbeat::route("/heartbeat"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
