use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed during {op}: {source}")]
    Http {
        op: &'static str,
        #[source]
        source: reqwest::Error,
    },

    #[error("API operation {op} failed: {message}")]
    Api { op: &'static str, message: String },

    #[error("failed to decode API response for {op}: {source}")]
    Decode {
        op: &'static str,
        #[source]
        source: serde_json::Error,
    },

    #[error("allocated GameServer response is missing required port '{0}'")]
    MissingPort(&'static str),

    #[error("timed out waiting for match {match_id} to {phase}")]
    Timeout {
        match_id: String,
        phase: &'static str,
    },

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("one or more matches failed; see summary.jsonl and match manifests")]
    MatchFailures,
}

impl Error {
    pub fn http(op: &'static str, source: reqwest::Error) -> Self {
        Self::Http { op, source }
    }

    pub fn decode(op: &'static str, source: serde_json::Error) -> Self {
        Self::Decode { op, source }
    }

    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
