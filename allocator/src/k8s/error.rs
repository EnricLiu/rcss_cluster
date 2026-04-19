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
    
    #[error("Failed to create Fleet[{fleet}], {source}")]
    CreateFleet {
        fleet: String,
        #[source]
        source: kube::Error,
    },
    
    #[error("Failed to delete Fleet, {0}")]
    DeleteFleet(#[source] kube::Error),

    #[error("Failed to select fleets, {0}")]
    SelectFleet(#[source] kube::Error),

    #[error("Fleet not found with the given labels")]
    FleetNotFound,

    #[error("Fleet[{fleet}]: labels do not match the expected labels. Expected: {expected}, Actual: {actual}")]
    FleetNotMatch {
        fleet: String,
        expected: String,
        actual: String,
    },

    #[error("Fleet[{fleet}] is not ready after waiting")]
    FleetNotReady {
        fleet: String,
    },

    #[error("Fleet[{fleet}] already exists")]
    FleetAlreadyExists {
        fleet: String,
    },

    #[error("Allocation error: {0}")]
    Allocation(#[from] super::allocation::AllocationError),

    #[error("{0}")]
    Custom(String),
}

impl Error {
    pub fn desc(&self) -> &'static str {
        match self {
            Error::CreateClient(_) => "K8sClientCreation",
            Error::UnsupportedVersion { .. } => "UnsupportedVersion",
            Error::InvalidFleetGS(_) => "InvalidFleetGS",
            Error::InvalidMetaData(_) => "InvalidMetaData",
            Error::CreateFleet{ .. } => "CreateFleet",
            Error::DeleteFleet(_) => "DeleteFleet",
            Error::Custom(_) => "Custom",
            Error::Allocation(e) => e.desc(),
            Error::SelectFleet(_) => "SelectFleet",
            Error::FleetNotFound => "FleetNotFound",
            Error::FleetNotMatch { .. } => "FleetNotMatch",
            Error::FleetAlreadyExists { .. } => "FleetAlreadyExists",
            Error::FleetNotReady { .. } => "FleetNotReady",
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
