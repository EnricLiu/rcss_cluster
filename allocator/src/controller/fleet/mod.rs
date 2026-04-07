mod drop;
mod create;
mod template;

use axum::Router;
use super::{Error, Result, AppState, Response};


pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(create::route("/create"))
        .merge(template::route("/template"))
        .merge(drop::route("/"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
