use super::Response;
use crate::service::cluster::error::Error;

impl Into<Response> for &Error {
    fn into(self) -> Response {
        use axum::http::StatusCode;

        match self {
            Error::RoomNotFound { room_id } => {
                let err: &'static str = self.into();
                Response::error(err, &format!("Room [{}] not found", room_id))
                    .with_status(StatusCode::NOT_FOUND)
            },
            Error::ClientNotFound { client_id } => {
                let err: &'static str = self.into();
                Response::error(err, &format!("Client [{}] not found", client_id))
                    .with_status(StatusCode::NOT_FOUND)
            },
            Error::ClientReleased { client_id } => {
                let err: &'static str = self.into();
                Response::error(err, &format!("Client [{}] already released", client_id))
            },
            Error::TeamNotFound { room_id, team_name } => {
                let err: &'static str = self.into();
                Response::error(err, &format!("Team [{}] not found in room [{}]", team_name, room_id))
                    .with_status(StatusCode::NOT_FOUND)
            }

            Error::Room { source, .. } => { source.into() },
            Error::Team { source, .. } => { source.into() },
            Error::Client { source, .. } => { source.into() },
        }
    }
}
impl Into<Response> for Error {
    fn into(self) -> Response {
        (&self).into()
    }
}