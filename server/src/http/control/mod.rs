#[cfg(feature = "standalone")]
mod restart;
mod shutdown;

use super::{AppState, Response};
use axum::Router;

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new();

    #[cfg(feature = "standalone")]
    let inner = inner.merge(restart::route("/restart"));
    
    let inner = inner
        .merge(shutdown::route("/shutdown"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
