use axum::extract::State;
use axum::{Router, routing};
use serde::Serialize;

use service::metrics::ServiceStatusInfo;

use super::{AppState, Response};

#[derive(Serialize, Debug)]
pub struct GetResponse {
    pub service: ServiceStatusInfo,
    pub conn_count: usize,
    #[cfg(feature = "agones")]
    pub agones: service::metrics::AgonesRuntimeInfo,
}

async fn get(State(state): State<AppState>) -> Response {
    let conn = state.conn_info().await;
    let service = state.service.status_info().await;

    Response::success(GetResponse {
        service,
        conn_count: conn.len(),
        #[cfg(feature = "agones")]
        agones: state.service.agones_runtime_info().await,
    })
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}
