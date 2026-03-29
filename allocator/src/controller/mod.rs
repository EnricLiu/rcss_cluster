pub mod allocate;
pub mod error;
pub mod health;
// pub mod response;
pub mod fleet;
mod gs;
mod state;

use axum::Router;

pub use error::{Error, Result};
pub use state::AppState;
pub use common::axum::response::Response;


pub fn route(path: &str, state: AppState) -> Router {
    let inner = Router::new()
        .merge(gs::route("/gs"))
        .merge(fleet::route("/fleet"))
        .merge(health::route("/ready"))
        .merge(health::route("/health"))
        .with_state(state);

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
