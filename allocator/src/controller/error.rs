use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Kubernetes API error: {0}")]
    K8s(#[from] kube::Error),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Error::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            Error::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            Error::K8s(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Kubernetes error: {}", err),
            ),
            Error::ResourceExhausted(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            Error::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
