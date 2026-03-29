use common::errors::BuilderError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create Kubernetes client, {0}")]
    CreateClient(#[source] kube::Error),

    #[error("Unsupported version {version} for resource {resource}. Supported versions: {supported:?}")]    
    UnsupportedVersion {
        version: u8,
        resource: &'static str,
        supported: &'static [u8],
    },
    
    #[error("Failed to parse metadata for the Fleet, {0}")]
    InvalidFleetGS(#[source] serde_json::Error),
    
    #[error("Failed to build the metadata, {0}")]
    InvalidMetaData(String),
    
    #[error("No gameservers available for allocation: {0}")]
    NoSuchGs(#[source] kube::Error),

    #[error("Failed to parse the gameserver allocation response, {0}")]
    GsaBadResponse(#[source] BuilderError),
    
    #[error("Failed to build the allocation, {0}")]
    GsaExhausted(String),
}

pub type Result<T> = std::result::Result<T, Error>;
