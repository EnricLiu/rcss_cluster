use std::path::PathBuf;
use std::sync::OnceLock;
use dashmap::DashMap;

use common::errors::{BuilderError, BuilderResult};
use common::types::Side;
use crate::config::RcssServerConfig;
use crate::model::coach::CoachModel;
use crate::model::player::PlayerModel;
use crate::declaration::{TeamDeclaration, Unum};


#[derive(Debug, Clone)]
pub struct TeamModel {
    pub declaration: TeamDeclaration,
    pub server: RcssServerConfig,
    pub log_root: Option<PathBuf>,
    pub players: OnceLock<DashMap<Unum, PlayerModel>>,
    pub coach: OnceLock<Option<CoachModel>>,
}

impl TeamModel {
    fn from(declaration: TeamDeclaration, server: RcssServerConfig, log_root: Option<PathBuf>) -> Self {
        Self { declaration, server, log_root, players: OnceLock::new(), coach: OnceLock::new() }
    }
    
    pub fn builder() -> TeamModelBuilder {
        TeamModelBuilder::default()
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.declaration.name.as_str()
    }

    #[inline]
    pub fn side(&self) -> Side {
        self.declaration.side
    }

    #[inline]
    pub fn server(&self) -> &RcssServerConfig {
        &self.server
    }

    pub fn players(&self) -> &DashMap<Unum, PlayerModel> {
        self.players.get_or_init(|| self.parse_players())
    }

    pub fn coach(&self) -> Option<&CoachModel> {
        self.coach.get_or_init(|| self.parse_coach()).as_ref()
    }

    fn parse_players(&self) -> DashMap<Unum, PlayerModel> {
        let map_fn = |(unum, p)| {
            let player = {
                let mut builder = PlayerModel::builder();
                builder
                    .with_declaration(p)
                    .with_team_side(self.side())
                    .with_team_name(self.name().to_string())
                    .with_server(self.server().player.clone())
                    .with_log_root(self.log_root.clone());

                builder.build_into().expect("Failed to build PlayerModel")
            };
            (unum, player)
        };

        self.declaration.players.clone().into_iter().map(map_fn).collect()
    }

    fn parse_coach(&self) -> Option<CoachModel> {
        self.declaration.coach.clone().map(|coach| {
            let mut builder = CoachModel::builder();
            builder
                .with_declaration(coach)
                .with_team_side(self.side())
                .with_team_name(self.name().to_string())
                .with_server(self.server().coach.clone())
                .with_log_root(self.log_root.clone());

            builder.build_into().expect("Failed to build CoachModel")
        })
    }
}

#[derive(Debug, Clone)]
pub struct TeamModelBuilder {
    declaration: Option<TeamDeclaration>,
    server: Option<RcssServerConfig>,
    log_root: Option<PathBuf>,
}

impl TeamModelBuilder {
    pub fn default() -> Self {
        Self { declaration: None, server: None, log_root: None }
    }

    pub fn with_declaration(&mut self, declaration: TeamDeclaration) -> &mut Self {
        self.declaration = Some(declaration);
        self
    }

    pub fn with_server(&mut self, server: RcssServerConfig) -> &mut Self {
        self.server = Some(server);
        self
    }

    pub fn with_log_root(&mut self, log_root: Option<PathBuf>) -> &mut Self {
        self.log_root = log_root;
        self
    }

    pub fn build(self) -> BuilderResult<TeamModel> {
        let declaration = self.declaration.ok_or(BuilderError::MissingField { field: "declaration" })?;
        let server = self.server.ok_or(BuilderError::MissingField { field: "server" })?;

        Ok(TeamModel::from(declaration, server, self.log_root))
    }
}
