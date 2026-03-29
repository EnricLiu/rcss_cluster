mod drop;
mod create;

use axum::Router;
use super::{Error, Result, AppState, Response};


pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(create::route("/create"))
        .merge(drop::route("/"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
