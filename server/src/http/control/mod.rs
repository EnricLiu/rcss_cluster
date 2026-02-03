#[cfg(feature = "standalone")]
mod restart;

use super::{AppState, Response};
use axum::Router;

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new();

    #[cfg(feature = "standalone")]
    let inner = inner.merge(restart::route("/restart"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
