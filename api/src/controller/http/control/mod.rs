mod restart;

use axum::Router;
use super::{AppState, Response};

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(restart::route("/restart"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
