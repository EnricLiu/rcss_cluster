use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use serde_json::Value;

use super::Response;
use super::{RoomError, ProxyError};
use crate::{room, self as proxy};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{value:?}")]
    Genetic {
        value: Option<Value>
    },

    #[error("I/O error: {source}")]
    IO {
        source: std::io::Error,
    },

    #[error("JSON error: {source}")]
    JSON {
        source: serde_json::Error,
    },

    #[error("Invalid argument: {value}")]
    InvalidArgument {
        value: String,
    },

    #[error("Room error: {0}")]
    Room(#[from] room::Error),
    
    #[error("Proxy error: {0}")]
    Proxy(#[from] proxy::Error),
}


impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::IO { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::JSON { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InvalidArgument { value: _ } => StatusCode::BAD_REQUEST,
            Error::Room(e) => RoomError(e).status_code(),
            Error::Proxy(e) => ProxyError(e).status_code(),
            Error::Genetic { value: _ } => StatusCode::OK,
        }
    }
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        match e {
            Error::Genetic { value } => Response::fail(StatusCode::OK, value),
            Error::Room(e) => RoomError(&e).into(),
            Error::Proxy(e) => ProxyError(&e).into(),
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
