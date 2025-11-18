mod cluster;

use std::backtrace::Backtrace;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use serde_json::Value;

use super::Response;

#[derive(snafu::Snafu, Debug)]
pub enum Error {
    #[snafu(display("{value:?}"))]
    Genetic {
        value: Option<Value>
    },

    #[snafu(display("I/O error: {source}"))]
    IO {
        source: std::io::Error,
        backtrace: Backtrace
    },

    #[snafu(display("JSON error: {source}"))]
    JSON {
        source: serde_json::Error,
        backtrace: Backtrace
    },

    #[snafu(display("Invalid argument: {value}"))]
    InvalidArgument {
        value: String,
    },

    Cluster {
        source: crate::service::cluster::Error,
    }
}


impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::IO { source: _, backtrace: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::JSON { source: _, backtrace: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InvalidArgument { value: _ } => StatusCode::BAD_REQUEST,
            Error::Cluster { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Genetic { value: _ } => StatusCode::OK,
        }
    }
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        match e {
            Error::Genetic { value } => Response::fail(StatusCode::OK, value),
            Error::Cluster { source } => source.into(),
            _ => Response::fail::<()>(e.status_code(), None),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> AxumResponse {
        let response = Response::from(self);
        response.into_response()
    }
}
