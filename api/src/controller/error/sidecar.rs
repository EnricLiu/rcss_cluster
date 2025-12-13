use super::Response;
use axum::http::StatusCode;

pub struct SidecarError<'a>(pub &'a sidecar::Error);

impl<'a> SidecarError<'a> {
    pub fn status_code(&self) -> StatusCode {
        match &self.0 {
            sidecar::Error::ServerNotRunning { status: _ } => StatusCode::OK,
            sidecar::Error::ServerStillRunningToRestart => StatusCode::OK,
        }
    }
}

impl<'a> From<SidecarError<'a>> for Response {
    fn from(value: SidecarError<'a>) -> Self {
        match &value.0 {
            sidecar::Error::ServerNotRunning { status: _ } => {
                Response::error("ServerNotRunning", "The sidecar server is not running.")
            }
            sidecar::Error::ServerStillRunningToRestart => Response::error(
                "ServerStillRunningToRestart",
                "The sidecar server is still running, try to call with `force=true` to ignore.",
            ),
        }
    }
}
