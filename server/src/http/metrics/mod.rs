mod status;
mod conn;
mod config;

use super::{AppState, Response};
use axum::Router;

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new();
    
    let inner = inner
        .merge(status::route("/health"))
        .merge(status::route("/status"))
        .merge(config::route("/config"))
        .merge(conn::route("/conn"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
