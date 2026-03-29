mod allocate;

use axum::Router;
use super::{Error, Result, AppState, Response};


pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(allocate::route("/allocate"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
