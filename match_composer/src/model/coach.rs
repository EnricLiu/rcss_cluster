use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use common::errors::{BuilderError, BuilderResult};
use common::types::Side;
use allocator::declaration::CoachKindDeclaration;

use crate::declaration::{CoachDeclaration, ImageDeclaration};
use super::ProcessModel;

#[derive(Debug, Clone)]
pub enum CoachModel {
    Helios(HeliosCoachModel),
    Ssp(SspCoachModel),
}

impl Deref for CoachModel {
    type Target = CoachBaseModel;

    fn deref(&self) -> &Self::Target {
        match self {
            CoachModel::Helios(params) => &params.base,
            CoachModel::Ssp(params) => &params.base,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum CoachKind {
    Helios,
    Ssp,
}

impl From<CoachKindDeclaration> for CoachKind {
    fn from(kind: CoachKindDeclaration) -> Self {
        match kind {
            CoachKindDeclaration::Helios => CoachKind::Helios,
            CoachKindDeclaration::Ssp => CoachKind::Ssp,
        }
    }
}

impl CoachKind {
    pub fn is_agent(&self) -> bool {
        matches!(self, CoachKind::Ssp)
    }

    pub fn is_bot(&self) -> bool {
        !self.is_agent()
    }
}

#[derive(Debug, Clone)]
pub struct CoachBaseModel {
    pub side: Side,
    pub team: String,
    pub kind: CoachKind,
    pub server: SocketAddr,
    pub image: ImageDeclaration,
    pub log_root: Option<PathBuf>,
}

impl ProcessModel for CoachBaseModel {
    fn image(&self) -> &ImageDeclaration {
        &self.image
    }

    fn log_dir(&self) -> Option<PathBuf> {
        self.log_root.clone()
    }

    fn log_file_name(&self) -> String {
        format!("{}-coach-stdio.log", self.team)
    }

    fn process_label(&self) -> String {
        format!("PolicyCoach(team={}, side={:?})", self.team, self.side)
    }
}

#[derive(Debug, Clone)]
pub struct HeliosCoachModel {
    base: CoachBaseModel,
}

impl Deref for HeliosCoachModel {
    type Target = CoachBaseModel;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl AsRef<CoachBaseModel> for HeliosCoachModel {
    fn as_ref(&self) -> &CoachBaseModel {
        &self.base
    }
}

#[derive(Debug, Clone)]
pub struct SspCoachModel {
    base: CoachBaseModel,
    pub grpc: SocketAddr,
}

impl Deref for SspCoachModel {
    type Target = CoachBaseModel;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl AsRef<CoachBaseModel> for SspCoachModel {
    fn as_ref(&self) -> &CoachBaseModel {
        &self.base
    }
}

impl CoachModel {
    pub fn builder() -> CoachModelBuilder {
        CoachModelBuilder::new()
    }
}

#[derive(Default)]
pub struct CoachModelBuilder {
    pub side: Option<Side>,
    pub team: Option<String>,
    pub kind: Option<CoachKind>,
    pub server: Option<SocketAddr>,
    pub image: Option<ImageDeclaration>,
    pub log_root: Option<PathBuf>,

    pub enable_log: bool,

    grpc: Option<SocketAddr>,
}

impl CoachModelBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_grpc(&mut self, grpc: SocketAddr) -> BuilderResult<&mut Self> {
        if  let Some(kind) = &self.kind &&
            !matches!(kind, CoachKind::Ssp) {
            return Err(BuilderError::InvalidField {
                field: "grpc",
                message: "Cannot set gRPC configuration for a non-SSP player".to_string(),
            });
        }

        self.with_kind(CoachKind::Ssp);
        self.grpc = Some(grpc);
        Ok(self)
    }


    pub fn with_declaration(&mut self, declaration: CoachDeclaration) -> &mut Self {
        match declaration {
            CoachDeclaration::Helios { base } => {
                self.kind = Some(CoachKind::Helios);
                self.image = Some(base.image);
                self.enable_log = base.log;
            },
            CoachDeclaration::Ssp { base, grpc } => {
                self.kind = Some(CoachKind::Ssp);
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

    pub fn with_kind(&mut self, kind: impl Into<CoachKind>) -> &mut Self {
        let kind = kind.into();

        match &kind {
            CoachKind::Helios => {
                if let Some(grpc) = &self.grpc {
                    log::warn!("Setting coach kind to Helios, but gRPC configuration is already set to {:?}. This configuration will be ignored.", grpc);
                }
            },
            CoachKind::Ssp => {

            },
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

    pub fn build_into(self) -> BuilderResult<CoachModel> {
        let side = self.side.ok_or(BuilderError::MissingField{ field: "side" })?;
        let team = self.team.ok_or(BuilderError::MissingField{ field: "team" })?;
        let kind = self.kind.ok_or(BuilderError::MissingField{ field: "kind" })?;
        let server = self.server.ok_or(BuilderError::MissingField{ field: "server" })?;
        let image = self.image.ok_or(BuilderError::MissingField{ field: "image" })?;
        let log_root = self.enable_log.then(|| {
            if self.log_root.is_none() {
                log::warn!("Logging is enabled for coach of team {}, but no log root directory is set. Logs will not be saved.", team);
            }
            self.log_root
        }).flatten();

        let base = CoachBaseModel { side, team, kind, server, image, log_root };

        match kind {
            CoachKind::Helios => Ok(CoachModel::Helios(HeliosCoachModel { base })),
            CoachKind::Ssp => {
                let grpc = self.grpc.ok_or(BuilderError::MissingField { field: "grpc" })?;
                Ok(CoachModel::Ssp(SspCoachModel { base, grpc }))
            }

        }
    }
}
