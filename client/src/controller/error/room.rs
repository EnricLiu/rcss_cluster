use axum::http::StatusCode;
use crate::room;
use super::Response;

pub struct RoomError<'a>(pub &'a room::Error);

impl<'a> RoomError<'a> {
    pub fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<'a> From<RoomError<'a>> for Response {
    fn from(value: RoomError<'a>) -> Self {
        Self::fail(value.status_code(), Some(value.0.to_string()))
    }
}
