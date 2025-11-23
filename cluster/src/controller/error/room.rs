use super::Response;
use crate::service::room::Error;

impl Into<Response> for &Error {
    fn into(self) -> Response {
        use axum::http::StatusCode;

        match self {
            _ => todo!()
        }
    }
}
impl Into<Response> for Error {
    fn into(self) -> Response {
        (&self).into()
    }
}