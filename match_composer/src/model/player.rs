use std::ops::Deref;
use std::net::SocketAddr;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use common::errors::{BuilderError, BuilderResult};
use common::types::Side;
use allocator::declaration::player::PlayerKind as PlayerKindDeclaration;

use crate::config::RcssServerConfig;
use crate::declaration::{ImageDeclaration, PlayerDeclaration, Unum};

#[derive(Debug, Clone)]
pub enum PlayerModel {
    Helios(HeliosPlayerModel),
    Ssp(SspPlayerModel),
}

impl Deref for PlayerModel {
    type Target = PlayerBaseModel;

    fn deref(&self) -> &Self::Target {
        match self {
            PlayerModel::Helios(params) => &params.base,
            PlayerModel::Ssp(params) => &params.base,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum PlayerKind {
    Helios,
    Ssp,
}

impl From<PlayerKindDeclaration> for PlayerKind {
    fn from(kind: PlayerKindDeclaration) -> Self {
        match kind {
            PlayerKindDeclaration::Helios => PlayerKind::Helios,
            PlayerKindDeclaration::Ssp => PlayerKind::Ssp,
        }
    }
}

impl PlayerKind {
    pub fn is_agent(&self) -> bool {
        matches!(self, PlayerKind::Ssp)
    }

    pub fn is_bot(&self) -> bool {
        !self.is_agent()
    }
}

impl PlayerModel {
    pub fn kind(&self) -> PlayerKind {
        match self {
            PlayerModel::Helios(_) => PlayerKind::Helios,
            PlayerModel::Ssp(_) => PlayerKind::Ssp,
        }
    }

    pub fn builder() -> PlayerModelBuilder {
        PlayerModelBuilder::new()
    }
}

#[derive(Debug, Clone)]
pub struct PlayerBaseModel {
    pub unum: Unum,
    pub side: Side,
    pub team: String,
    pub kind: PlayerKind,
    pub goalie: bool,
    pub server: SocketAddr,
    pub image: ImageDeclaration,
    pub log_root: Option<PathBuf>,
}

impl PlayerBaseModel {
    pub fn new(
        unum: Unum,
        side: Side,
        team: String,
        kind: PlayerKind,
        goalie: bool,
        server: SocketAddr,
        image: ImageDeclaration,
        log_root: Option<PathBuf>,
    ) -> Self {
        PlayerBaseModel {
            unum,
            side,
            team,
            kind,
            goalie,
            server,
            image,
            log_root,
        }
    }
}


#[derive(Debug, Clone)]
pub struct HeliosPlayerModel {
    base: PlayerBaseModel,
}

impl Deref for HeliosPlayerModel {
    type Target = PlayerBaseModel;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl AsRef<PlayerBaseModel> for HeliosPlayerModel {
    fn as_ref(&self) -> &PlayerBaseModel {
        &self.base
    }
}

#[derive(Debug, Clone)]
pub struct SspPlayerModel {
    base: PlayerBaseModel,
    pub grpc: SocketAddr,
}

impl Deref for SspPlayerModel {
    type Target = PlayerBaseModel;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl AsRef<PlayerBaseModel> for SspPlayerModel {
    fn as_ref(&self) -> &PlayerBaseModel {
        &self.base
    }
}

#[derive(Default)]
pub struct PlayerModelBuilder {
    pub unum: Option<Unum>,
    pub side: Option<Side>,
    pub team: Option<String>,
    pub kind: Option<PlayerKind>,
    pub goalie: Option<bool>,
    pub server: Option<SocketAddr>,
    pub image: Option<ImageDeclaration>,
    pub log_root: Option<PathBuf>,

    pub enable_log: bool,

    grpc: Option<SocketAddr>,
}

impl PlayerModelBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_grpc(&mut self, grpc: SocketAddr) -> BuilderResult<&mut Self> {
        if  let Some(kind) = &self.kind &&
            !matches!(kind, PlayerKind::Ssp) {
            return Err(BuilderError::InvalidField {
                field: "grpc",
                message: "Cannot set gRPC configuration for a non-SSP player".to_string(),
            });
        }

        self.with_kind(PlayerKind::Ssp);
        self.grpc = Some(grpc);
        Ok(self)
    }

    pub fn with_declaration(
        &mut self,
        declaration: PlayerDeclaration,
    ) -> &mut Self {
        let kind = declaration.kind();

        let base = match declaration {
            PlayerDeclaration::Ssp { grpc, base } => {
                self.with_grpc(grpc.into()).expect("Failed to set gRPC configuration for SSP player");
                base
            },
            PlayerDeclaration::Helios { base } => base,
        };

        self.with_unum(base.unum);
        self.with_kind(kind);
        self.with_goalie(base.goalie);
        self.with_image(base.image);
        self.enable_log = base.log;

        self
    }

    pub fn with_unum(&mut self, unum: Unum) -> &mut Self {
        self.unum = Some(unum);
        self
    }

    pub fn with_kind(&mut self, kind: impl Into<PlayerKind>) -> &mut Self {
        let kind = kind.into();
        
        match &kind {
            PlayerKind::Helios => {
                if let Some(grpc) = &self.grpc {
                    log::warn!("Setting player kind to Helios, but gRPC configuration is already set to {:?}. This configuration will be ignored.", grpc);
                }
            },
            PlayerKind::Ssp => {

            },
        }
        self.kind = Some(kind);

        self
    }

    pub fn with_image(&mut self, image: ImageDeclaration) -> &mut Self {
        self.image = Some(image);
        self
    }

    pub fn with_goalie(&mut self, goalie: bool) -> &mut Self {
        self.goalie = Some(goalie);
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

    pub fn with_server(&mut self, server: SocketAddr) -> &mut Self {
        self.server = Some(server);
        self
    }

    pub fn with_log_root(&mut self, log_root: Option<PathBuf>) -> &mut Self {
        self.log_root = log_root;
        self
    }

    pub fn build_into(self) -> BuilderResult<PlayerModel> {
        let unum = self.unum.ok_or(BuilderError::MissingField { field: "unum" })?;
        let side = self.side.ok_or(BuilderError::MissingField{ field: "side" })?;
        let team = self.team.ok_or(BuilderError::MissingField{ field: "team" })?;
        let kind = self.kind.ok_or(BuilderError::MissingField{ field: "kind" })?;
        let goalie = self.goalie.ok_or(BuilderError::MissingField{ field: "goalie" })?;
        let server = self.server.ok_or(BuilderError::MissingField{ field: "server" })?;
        let image = self.image.ok_or(BuilderError::MissingField{ field: "image" })?;
        let log_root = self.enable_log.then(|| {
            if self.log_root.is_none() {
                log::warn!("Logging is enabled for player {}, but no log root directory is set. Logs will not be saved.", unum);
            }
            self.log_root
        }).flatten();

        let base = PlayerBaseModel::new(unum, side, team, kind, goalie, server, image, log_root);

        match kind {
            PlayerKind::Helios => Ok(PlayerModel::Helios(HeliosPlayerModel { base })),
            PlayerKind::Ssp => {
                let grpc = self.grpc.ok_or(BuilderError::MissingField { field: "grpc" })?;
                Ok(PlayerModel::Ssp(SspPlayerModel { base, grpc }))
            }
        }
    }
}

