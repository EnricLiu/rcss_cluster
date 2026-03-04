pub mod allocate;
pub mod error;
pub mod health;
pub mod response;

use axum::{
    routing::{get, post},
    Router,
};

use allocate::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .route("/api/v1/allocate", post(allocate::allocate))
        .with_state(state)
}
