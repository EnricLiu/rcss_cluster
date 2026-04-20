use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use serde_json::json;
use common::axum::response::Response;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::Conflict(_)   => StatusCode::CONFLICT,
            Error::Internal(_)   => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> AxumResponse {
        let status = self.status_code();
        let body = Json(json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Response {
        Response::fail(err.status_code(), Some(json!({ "error": err.to_string() })))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
