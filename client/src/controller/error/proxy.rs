use axum::http::StatusCode;
use crate as proxy;

use super::Response;
use super::RoomError;

pub struct ProxyError<'a>(pub &'a proxy::Error);

impl<'a> ProxyError<'a> {
    pub fn status_code(&self) -> StatusCode {
        match self.0 {
            proxy::Error::RoomNotFound { .. } => StatusCode::NOT_FOUND,
            proxy::Error::RoomDropDangled { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            proxy::Error::RoomDropRetrieved { .. } => StatusCode::OK,
            proxy::Error::RoomDropped { .. } => StatusCode::OK,
            proxy::Error::Room(room) => RoomError(room).status_code(),
        }
    }
}

impl<'a> From<ProxyError<'a>> for Response {
    fn from(value: ProxyError<'a>) -> Self {
        use proxy::Error::*;
        let err = match value.0 {
            RoomNotFound { .. } => "RoomNotFound",
            RoomDropDangled { .. } => "RoomDropDangled",
            RoomDropRetrieved { .. } => "RoomDropRetrieved",
            RoomDropped { .. } => "RoomDropped",
            Room(_) => "Room",
        };
        
        let desc = value.0.to_string();
        Self::error(err, &desc).with_status(value.status_code())
    }
}
