#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// HTTP connection failed (e.g. connection refused, DNS error)
    #[error("HTTP connection failed: {0}")]
    Connection(#[source] reqwest::Error),

    /// Server returned a non-2xx HTTP status
    #[error("HTTP request failed with status {status}: {body}")]
    RequestFailed { status: u16, body: String },

    /// Failed to deserialize the response body into common response model from reqwest
    #[error("Failed to deserialize response text into common::Response: {source}")]
    ReqwestDesFailed {
        #[source]
        source: reqwest::Error,
        model: &'static str,
    },

    #[error("Failed to deserialize response value into {model}: {source}")]
    SerdeDesFailed {
        #[source]
        source: serde_json::Error,
        model: &'static str,
    },
}
