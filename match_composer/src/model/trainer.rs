use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;

use allocator::declaration::{CoachDeclaration, CoachKindDeclaration};
use common::errors::{BuilderError, BuilderResult};
use common::types::Side;
use serde::{Deserialize, Serialize};

use crate::declaration::ImageDeclaration;
use super::ProcessModel;

#[derive(Debug, Clone)]
pub enum TrainerModel {
    Helios(HeliosTrainerModel),
    Ssp(SspTrainerModel),
}

impl Deref for TrainerModel {
    type Target = TrainerBaseModel;

    fn deref(&self) -> &Self::Target {
        match self {
            TrainerModel::Helios(params) => &params.base,
            TrainerModel::Ssp(params) => &params.base,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum TrainerKind {
    Helios,
    Ssp,
}

impl From<CoachKindDeclaration> for TrainerKind {
    fn from(kind: CoachKindDeclaration) -> Self {
        match kind {
            CoachKindDeclaration::Helios => TrainerKind::Helios,
            CoachKindDeclaration::Ssp => TrainerKind::Ssp,
        }
    }
}

impl TrainerKind {
    pub fn is_agent(&self) -> bool {
        matches!(self, TrainerKind::Ssp)
    }

    pub fn is_bot(&self) -> bool {
        !self.is_agent()
    }
}

#[derive(Debug, Clone)]
pub struct TrainerBaseModel {
    pub side: Side,
    pub team: String,
    pub kind: TrainerKind,
    pub server: SocketAddr,
    pub image: ImageDeclaration,
    pub log_root: Option<PathBuf>,
}

impl ProcessModel for TrainerBaseModel {
    fn image(&self) -> &ImageDeclaration {
        &self.image
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.log_root.clone()
    }

    fn log_file_name(&self) -> String {
        format!("{}-trainer-stdio.log", self.team)
    }

    fn process_label(&self) -> String {
        format!("PolicyTrainer(team={}, side={:?})", self.team, self.side)
    }
}

#[derive(Debug, Clone)]
pub struct HeliosTrainerModel {
    base: TrainerBaseModel,
}

impl Deref for HeliosTrainerModel {
    type Target = TrainerBaseModel;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl AsRef<TrainerBaseModel> for HeliosTrainerModel {
    fn as_ref(&self) -> &TrainerBaseModel {
        &self.base
    }
}

#[derive(Debug, Clone)]
pub struct SspTrainerModel {
    base: TrainerBaseModel,
    pub grpc: SocketAddr,
}

impl Deref for SspTrainerModel {
    type Target = TrainerBaseModel;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl AsRef<TrainerBaseModel> for SspTrainerModel {
    fn as_ref(&self) -> &TrainerBaseModel {
        &self.base
    }
}

impl TrainerModel {
    pub fn builder() -> TrainerModelBuilder {
        TrainerModelBuilder::new()
    }
}

#[derive(Default)]
pub struct TrainerModelBuilder {
    pub side: Option<Side>,
    pub team: Option<String>,
    pub kind: Option<TrainerKind>,
    pub server: Option<SocketAddr>,
    pub image: Option<ImageDeclaration>,
    pub log_root: Option<PathBuf>,

    pub enable_log: bool,

    grpc: Option<SocketAddr>,
}

impl TrainerModelBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_grpc(&mut self, grpc: SocketAddr) -> BuilderResult<&mut Self> {
        if let Some(kind) = &self.kind &&
            !matches!(kind, TrainerKind::Ssp) {
            return Err(BuilderError::InvalidField {
                field: "grpc",
                message: "Cannot set gRPC configuration for a non-SSP trainer".to_string(),
            });
        }

        self.with_kind(TrainerKind::Ssp);
        self.grpc = Some(grpc);
        Ok(self)
    }

    pub fn with_declaration(&mut self, declaration: CoachDeclaration) -> &mut Self {
        match declaration {
            CoachDeclaration::Helios { base } => {
                self.kind = Some(TrainerKind::Helios);
                self.image = Some(base.image);
                self.enable_log = base.log;
            },
            CoachDeclaration::Ssp { base, grpc } => {
                self.kind = Some(TrainerKind::Ssp);
                self.image = Some(base.image);
                self.enable_log = base.log;
                self.grpc = Some(grpc.into());
            }
        }
        self
    }

    pub fn with_team_name(&mut self, team: String) -> &mut Self {
        self.team = Some(team);
        self
    }

    pub fn with_team_side(&mut self, side: Side) -> &mut Self {
        self.side = Some(side);
        self
    }

    pub fn with_kind(&mut self, kind: impl Into<TrainerKind>) -> &mut Self {
        let kind = kind.into();

        match &kind {
            TrainerKind::Helios => {
                if let Some(grpc) = &self.grpc {
                    log::warn!("Setting trainer kind to Helios, but gRPC configuration is already set to {:?}. This configuration will be ignored.", grpc);
                }
            },
            TrainerKind::Ssp => {},
        }
        self.kind = Some(kind);

        self
    }

    pub fn with_server(&mut self, server: SocketAddr) -> &mut Self {
        self.server = Some(server);
        self
    }

    pub fn with_log_root(&mut self, log_root: Option<PathBuf>) -> &mut Self {
        self.log_root = log_root;
        self
    }

    pub fn build_into(self) -> BuilderResult<TrainerModel> {
        let side = self.side.ok_or(BuilderError::MissingField{ field: "side" })?;
        let team = self.team.ok_or(BuilderError::MissingField{ field: "team" })?;
        let kind = self.kind.ok_or(BuilderError::MissingField{ field: "kind" })?;
        let server = self.server.ok_or(BuilderError::MissingField{ field: "server" })?;
        let image = self.image.ok_or(BuilderError::MissingField{ field: "image" })?;
        let log_root = self.enable_log.then(|| {
            if self.log_root.is_none() {
                log::warn!("Logging is enabled for trainer of team {}, but no log root directory is set. Logs will not be saved.", team);
            }
            self.log_root
        }).flatten();

        let base = TrainerBaseModel { side, team, kind, server, image, log_root };

        match kind {
            TrainerKind::Helios => Ok(TrainerModel::Helios(HeliosTrainerModel { base })),
            TrainerKind::Ssp => {
                let grpc = self.grpc.ok_or(BuilderError::MissingField { field: "grpc" })?;
                Ok(TrainerModel::Ssp(SspTrainerModel { base, grpc }))
            }
        }
    }
}
