use std::path::{Path, PathBuf};
use log::info;
use tokio::process::Child;
use common::types::Side;

use crate::config::{MatchComposerConfig, ServerConfig};
use crate::policy::PolicyRegistry;
use crate::team;
use crate::team::{Team, TeamMeta};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposerStatus {
    Idle,
    Booting,
    Running,
    ShuttingDown,
}

pub struct MatchComposer {
    pub config: MatchComposerConfig,
    pub log_root: Option<PathBuf>,

    registry: PolicyRegistry,
    server_process: Option<Child>,

    pub allies: Team,
    pub opponents: Team,
}

impl MatchComposer {
    pub fn new(config: MatchComposerConfig, registry_path: impl AsRef<Path>) -> Result<Self> {
        let registry = PolicyRegistry::new(registry_path);
        log::debug!("{:?}", registry.images.providers().and_then(|i| Some(i.collect::<Vec<_>>())));

        let allies = Team::new(config.allies.clone());
        let opponents = Team::new(config.opponents.clone());
        let log_root = config.log_root.clone();
        Ok(Self {
            config,
            registry,
            server_process: None,
            allies,
            opponents,
            log_root,
        })
    }
    
    
    pub async fn shutdown(&mut self) {
        self.allies.shutdown().await;
        self.opponents.shutdown().await;
        if let Some(mut proc) = self.server_process.take() {
            let _ = proc.kill().await;
        }
    }

    pub async fn spawn_players(&mut self) -> Result<()> {
        self.allies.spawn(&self.registry).await?;
        info!("Allies spawned successfully, {:?}", self.allies.meta());
        self.opponents.spawn(&self.registry).await?;
        info!("Opponents spawned successfully, {:?}", self.opponents.meta());
        Ok(())
    }

    pub async fn wait(&mut self) -> Result<()> {
        self.allies.wait().await?;
        self.opponents.wait().await?;
        Ok(())
    }

    pub fn teams(&self) -> (TeamMeta, TeamMeta) {
        let left = self.allies.meta();
        let right = self.opponents.meta();
        (left, right)
    }

    pub fn team(&self, side: Side) -> TeamMeta {
        match side {
            Side::LEFT => self.allies.meta(),
            Side::RIGHT => self.opponents.meta(),
            Side::NEUTRAL => unreachable!("Side should only be LEFT or RIGHT, cannot be NEUTRAL"),
        }
    }

    pub fn rcss_conn(&self) -> ServerConfig {
        self.config.server.clone()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Team Error: {0}")]
    Team(#[from] team::Error)
}

pub type Result<T> = std::result::Result<T, Error>;

