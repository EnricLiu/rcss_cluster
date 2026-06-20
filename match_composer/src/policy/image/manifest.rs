use std::collections::BTreeMap;
use std::path::{Component, PathBuf};

use serde::Deserialize;
use tokio::process::Command;

use crate::model::ImageInfo;
use super::PolicyImage;

pub const MANIFEST_FILE: &str = "metadata.json";
pub const SUPPORTED_SCHEMA_VERSION: u16 = 1;
pub const DEFAULT_READY_STDOUT: &str = "init ok.";

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ImageRole {
    Player,
    Coach,
    Trainer,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Ssp,
    Helios,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ImageRoleManifest {
    pub entrypoint: PathBuf,
    #[serde(default)]
    pub ready_stdout_contains: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ImageManifest {
    pub schema_version: u16,
    pub id: String,
    pub format: ImageFormat,
    #[serde(default)]
    pub description: Option<String>,
    pub roles: BTreeMap<ImageRole, ImageRoleManifest>,
}

#[derive(Debug)]
pub struct ManifestImage {
    image: ImageInfo,
    manifest: ImageManifest,
}

impl ManifestImage {
    pub fn load(image: ImageInfo) -> Result<Self, ManifestLoadError> {
        let path = image.path.join(MANIFEST_FILE);
        if !path.exists() {
            return Err(ManifestLoadError::Missing);
        }

        let raw = std::fs::read_to_string(&path)
            .map_err(|source| ManifestLoadError::Read { path: path.clone(), source })?;
        let manifest: ImageManifest = serde_json::from_str(&raw)
            .map_err(|source| ManifestLoadError::Parse { path: path.clone(), source })?;

        validate_manifest(&image, &manifest)?;

        Ok(Self { image, manifest })
    }
}

impl PolicyImage for ManifestImage {
    fn image(&self) -> &ImageInfo {
        &self.image
    }

    fn format(&self) -> ImageFormat {
        self.manifest.format
    }

    fn command(&self, role: ImageRole) -> Option<Command> {
        self.manifest
            .roles
            .get(&role)
            .map(|role| Command::new(self.image.path.join(&role.entrypoint)))
    }

    fn ready_stdout_contains(&self, role: ImageRole) -> Option<&str> {
        self.manifest
            .roles
            .get(&role)
            .and_then(|role| role.ready_stdout_contains.as_deref())
            .or(Some(DEFAULT_READY_STDOUT))
    }
}

fn validate_manifest(image: &ImageInfo, manifest: &ImageManifest) -> Result<(), ManifestLoadError> {
    if manifest.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(ManifestLoadError::UnsupportedSchemaVersion {
            actual: manifest.schema_version,
            expected: SUPPORTED_SCHEMA_VERSION,
        });
    }

    let expected_id = image.to_raw();
    if manifest.id != expected_id {
        return Err(ManifestLoadError::IdMismatch {
            actual: manifest.id.clone(),
            expected: expected_id,
        });
    }

    if let Some(description) = &manifest.description && description.contains('\0') {
        return Err(ManifestLoadError::InvalidField {
            field: "description",
            reason: "must not contain NUL",
        });
    }

    for (role, role_manifest) in &manifest.roles {
        validate_entrypoint(image, &role_manifest.entrypoint, *role)?;
    }

    Ok(())
}

fn validate_entrypoint(
    image: &ImageInfo,
    entrypoint: &PathBuf,
    role: ImageRole,
) -> Result<(), ManifestLoadError> {
    let mut has_normal_component = false;
    for component in entrypoint.components() {
        match component {
            Component::Normal(_) => has_normal_component = true,
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ManifestLoadError::UnsafeEntrypoint {
                    role,
                    entrypoint: entrypoint.clone(),
                });
            }
        }
    }

    if !has_normal_component {
        return Err(ManifestLoadError::UnsafeEntrypoint {
            role,
            entrypoint: entrypoint.clone(),
        });
    }

    let full_path = image.path.join(entrypoint);
    if !full_path.is_file() {
        return Err(ManifestLoadError::EntrypointMissing {
            role,
            path: full_path,
        });
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ManifestLoadError {
    #[error("manifest is missing")]
    Missing,
    #[error("failed to read manifest {path:?}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse manifest {path:?}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unsupported manifest schema version {actual}, expected {expected}")]
    UnsupportedSchemaVersion { actual: u16, expected: u16 },
    #[error("manifest id mismatch, actual {actual}, expected {expected}")]
    IdMismatch { actual: String, expected: String },
    #[error("invalid manifest field {field}: {reason}")]
    InvalidField {
        field: &'static str,
        reason: &'static str,
    },
    #[error("unsafe entrypoint {entrypoint:?} for role {role:?}")]
    UnsafeEntrypoint { role: ImageRole, entrypoint: PathBuf },
    #[error("entrypoint for role {role:?} does not exist: {path:?}")]
    EntrypointMissing { role: ImageRole, path: PathBuf },
}
