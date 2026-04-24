use std::sync::{Arc, OnceLock};
use std::time::Duration;
use chrono::{DateTime, Utc};
use log::info;

use tokio::sync::{watch, RwLock};

use common::types::Side;

use crate::team::{self, Team, TeamStatus};
use crate::metadata::MetaData;
use crate::policy::PolicyRegistry;
use crate::declaration::HostPort;
use crate::info::game::GameStatusInfo;
use crate::info::GameInfo;

use crate::config::{MatchComposerConfig, RcssServerConfig};

pub struct MatchComposer {
    pub config: MatchComposerConfig,

    match_meta: OnceLock<RwLock<Arc<MetaData>>>,

    registry: PolicyRegistry,
}

impl MatchComposer {
    pub fn new(
        config: MatchComposerConfig,
    ) -> Result<Self> {
        let registry = PolicyRegistry::new(&config.registry_path);

        Ok(Self {
            config,
            registry,
            match_meta: OnceLock::new(),
        })
    }

    fn is_init(&self) -> bool {
        self.match_meta.get().is_some()
    }

    fn match_data_unchecked(&self) -> &RwLock<Arc<MetaData>> {
        self.match_meta.get().unwrap()
    }

    pub async fn set_meta(&self, meta: MetaData) {
        let meta = Arc::new(meta);
        let lock = self.match_meta.get_or_init(|| RwLock::new(meta.clone()));
        *lock.write().await = meta;
    }

    pub async fn make_match(&self, log_name: impl AsRef<str>) -> Result<Match> {
        if !self.is_init() {
            return Err(Error::Team(team::Error::NoMatchMetaData));
        }

        let server = self.config.server.clone();
        let log = self.config.log_root.as_ref().map(|p| p.join(log_name.as_ref()));

        let meta = self.match_data_unchecked().read().await.clone();
        let declared = meta.as_model();
        let (team_l, team_r) = {
            let (team_l, team_r) = declared.teams(server.clone(), log);
            (Team::new(team_l), Team::new(team_r))
        };

        let game = {
            let mut game = Match::new(
                server,
                meta,
                team_l,
                team_r,
            );

            match self.config.concurrent_team_spawn {
                true => game.spawn_concurrent(
                    &self.registry,
                    self.config.player_spawn_delay,
                    self.config.team_spawn_delay,
                ).await?,
                
                false => game.spawn(
                    &self.registry,
                    self.config.player_spawn_delay,
                    self.config.team_spawn_delay,
                ).await?,
            }
            
            game
        };
        
        Ok(game)
    }
}

pub struct Match {
    pub rcsss: RcssServerConfig,
    pub config: Arc<MetaData>,
    pub team_l: Team,
    pub team_r: Team,
    pub status: watch::Receiver<GameStatusInfo>,
    
    status_tx: watch::Sender<GameStatusInfo>,
}

impl Match {
    pub fn new(
        rcsss: RcssServerConfig,
        config: Arc<MetaData>,
        team_l: Team,
        team_r: Team,
    ) -> Self {
        let (status_tx, status) = watch::channel(GameStatusInfo::Idle);
        
        Self {
            rcsss,
            config,
            team_l,
            team_r,
            status,
            status_tx,
        }
    }

    pub async fn spawn(
        &mut self,
        registry: &PolicyRegistry,
        player_delay: Duration,
        team_delay: Duration
    ) -> Result<()> {
        self.team_l.spawn(registry, player_delay).await?;
        info!("Team L spawned successfully, {:?}", self.team_l.info());
        tokio::time::sleep(team_delay).await;
        self.team_r.spawn(registry, player_delay).await?;
        info!("Team R spawned successfully, {:?}", self.team_r.info());
        Ok(())
    }

    pub async fn spawn_concurrent(
        &mut self,
        registry: &PolicyRegistry,
        player_delay: Duration,
        team_delay: Duration
    ) -> Result<()> {
        let spawn_l = self.team_l.spawn(registry, player_delay);
        let spawn_r = async {
            tokio::time::sleep(team_delay).await;
            self.team_r.spawn(registry, player_delay).await
        };
        
        tokio::select! {
            res = spawn_l => {
                res?;
                info!("Team L spawned successfully, {:?}", self.team_l.info());
            }
            res = spawn_r => {
                res?;
                info!("Team R spawned successfully, {:?}", self.team_r.info());
            },
        }
        
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.team_l.shutdown().await;
        self.team_r.shutdown().await;

        let _ = self.wait().await?;

        Ok(())
    }

    pub async fn team_watch_task(team: &Team) {
        let mut watch = team.status_watch();
        let _ = watch.wait_for(|s| s.is_finished()).await;
    }

    pub async fn wait(&mut self) -> Result<(TeamStatus, TeamStatus)> {
        let res = tokio::select! {
            res_l = self.team_l.wait() => {
                self.team_r.shutdown().await;
                let res_r = self.team_r.wait().await;
                (res_l?, res_r?)
            },
            res_r = self.team_r.wait() => {
                self.team_l.shutdown().await;
                let res_l = self.team_l.wait().await;
                (res_l?, res_r?)
            }
        };

        Ok(res)
    }

    pub fn team(&self, side: Side) -> &Team {
        match side {
            Side::LEFT => &self.team_l,
            Side::RIGHT => &self.team_r,
            _ => panic!("invalid side"),
        }
    }

    pub fn info(&self) -> GameInfo {
        GameInfo {
            rcss: self.rcsss.control.clone().into(),
            status: self.status_now(),
            team_l: self.team_l.info(),
            team_r: self.team_r.info(),
        }
    }
    
    pub fn status_now(&self) -> GameStatusInfo {
        self.status.borrow().clone()
    }
    
    pub fn status_watch(&self) -> watch::Receiver<GameStatusInfo> {
        self.status.clone()
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Team Error: {0}")]
    Team(#[from] team::Error)
}

pub type Result<T> = std::result::Result<T, Error>;

