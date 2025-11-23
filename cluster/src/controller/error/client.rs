use super::Response;
use crate::service::client::Error;

impl Into<Response> for &Error {
    fn into(self) -> Response {
        use axum::http::StatusCode;

        match self {
            Error::ChannelSend { .. }
                | Error::ChannelClosed { .. }
                | Error::TaskJoin { .. }
                | Error::Udp { .. }
            => Response::code(StatusCode::INTERNAL_SERVER_ERROR),

            Error::TimeoutInitReq { .. }
                | Error::TimeoutInitResp { .. }
            => {
                Response::error(self.into(), &self.to_string())
            },
            _ => todo!(),
        }
    }
}
impl Into<Response> for Error {
    fn into(self) -> Response {
        (&self).into()
    }
}