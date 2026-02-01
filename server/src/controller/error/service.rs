use super::Response;
use axum::http::StatusCode;
use service::Error;

pub struct ServiceError<'a>(pub &'a Error);

impl<'a> ServiceError<'a> {
    pub fn status_code(&self) -> StatusCode {
        match &self.0 {
            Error::ServerNotRunning { status: _ } => StatusCode::OK,
            Error::ServerStillRunningToSpawn => StatusCode::OK,
            Error::Timeout { op: _ } => StatusCode::REQUEST_TIMEOUT,
            Error::ProcessFailedToShutdown => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ProcessSpawnFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::TrainerCommandFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::StatusChannelClosed => StatusCode::INTERNAL_SERVER_ERROR,
            #[cfg(feature = "agones")]
            Error::AgonesSdkFailToConnect(_) => unreachable!(),
            #[cfg(feature = "agones")]
            Error::AgonesSdkReadyFailed(_) => unreachable!(),
            #[cfg(feature = "agones")]
            Error::AgonesSdkShutdownFailed(_) => unreachable!(),
        }
    }
}

impl<'a> From<ServiceError<'a>> for Response {
    fn from(value: ServiceError<'a>) -> Self {
        match &value.0 {
            Error::ServerNotRunning { status: _ } => {
                Response::error("ServerNotRunning", "The process server is not running.")
            }
            Error::ServerStillRunningToSpawn => Response::error(
                "ServerStillRunningToSpawn", &value.0.to_string(),
            ),
            Error::Timeout { op: _ } => {
                Response::error("Timeout", &value.0.to_string())
            },
            Error::ProcessFailedToShutdown => {
                Response::error(
                    "ProcessFailedToShutdown",
                    "Failed to shutdown process due to internal error."
                )
            },
            Error::ProcessSpawnFailed(_) => {
                Response::error(
                    "ProcessSpawnFailed",
                    "Failed to spawn process due to internal error."
                )
            },
            Error::TrainerCommandFailed(_) => {
                Response::error(
                    "TrainerCommandFailed",
                    "Failed to send command to trainer process due to internal error."
                )
            },
            Error::StatusChannelClosed => {
                Response::error(
                    "StatusChannelClosed",
                    "The status channel has been closed unexpectedly."
                )
            },

            #[cfg(feature = "agones")]
            Error::AgonesSdkFailToConnect(_) => {
                Response::error(
                    "AgonesSdkFailToConnect",
                    "TODO", // TODO
                )
            },

            #[cfg(feature = "agones")]
            Error::AgonesSdkReadyFailed(_) => {
                Response::error(
                    "AgonesSdkReadyFailed",
                    "TODO", // TODO
                )
            },

            #[cfg(feature = "agones")]
            Error::AgonesSdkShutdownFailed(_) => {
                Response::error(
                    "AgonesSdkShutdownFailed",
                    "TODO", // TODO
                )
            },
        }
    }
}
