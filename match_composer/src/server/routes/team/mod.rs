use axum::Router;
use super::{AppState, Result, Error};

pub mod status;

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(status::route("/status"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}