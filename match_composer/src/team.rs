use std::ops::{Deref, DerefMut};
use std::time::Duration;

use tokio::sync::watch;
use dashmap::{DashMap, DashSet};
use crate::model::TeamModel;
use crate::player::{Player, PolicyPlayer};
use crate::policy::PolicyRegistry;
use crate::declarations::{ImageDeclaration, Unum};
use crate::info::{PlayerInfo, TeamInfo};
pub use crate::info::TeamStatusInfo as TeamStatus;

pub const SPAWN_DURATION: Duration = Duration::from_millis(100);


#[derive(Debug)]
pub struct PlayerWrap(Box<dyn Player>);
impl<P: Player> From<P> for PlayerWrap {
    fn from(player: P) -> Self {
        Self(Box::new(player))
    }
}
impl Deref for PlayerWrap {
    type Target = Box<dyn Player>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerWrap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PlayerWrap {
    pub fn info(&self) -> PlayerInfo {
        let model = self.model();
        PlayerInfo {
            unum: model.unum,
            kind: model.kind,
            image: model.image.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Team {
    pub config: TeamModel,

    status_tx: watch::Sender<TeamStatus>,
    status_rx: watch::Receiver<TeamStatus>,
    players: DashMap<Unum, PlayerWrap>,
    agents: DashSet<Unum>,
}

impl Team {
    pub fn new(config: TeamModel) -> Self {
        let (status_tx, status_rx) = watch::channel(TeamStatus::Idle);
        Self {
            config,
            status_tx,
            status_rx,
            players: DashMap::new(),
            agents: DashSet::new(),
        }
    }

    pub fn status_now(&self) -> TeamStatus {
        self.status_rx.borrow().clone()
    }

    pub fn status_watch(&self) -> watch::Receiver<TeamStatus> {
        self.status_rx.clone()
    }

    pub async fn spawn(
        &mut self,
        registry: &PolicyRegistry,
    ) -> Result<()> {
        if !self.status_tx.borrow().is_finished() {
            return Err(Error::NotFinished);
        }
        self.status_tx.send(TeamStatus::Starting)
            .map_err(|_| Error::ChannelClosed { ch_name: "TeamStatus" })?;

        let mut players = self.config.players().clone()
            .into_iter().map(|(_, p)| p).collect::<Vec<_>>();

        players.sort_by_key(|p| p.unum);

        let mut interval = tokio::time::interval(SPAWN_DURATION);
        for player in players {
            let unum = player.unum;
            let policy = registry.fetch(player).map_err(|player| {
                let err = Error::PolicyNotFound { image: player.image.clone() };
                self.status_tx.send(TeamStatus::Error(err.clone())).ok();
                err
            })?;

            if policy.info().kind.is_agent() {
                if self.agents.contains(&unum) { continue }
                self.agents.insert(unum);
            }

            let player = PolicyPlayer::new(policy);
            player.spawn().await.map_err(|e|Error::SpawnPlayer(format!("{e:?}")))?;
            self.players.insert(unum, player.into());

            interval.tick().await;
        }

        if let Err(e) = self.status_tx.send(TeamStatus::Running) {
            self.shutdown().await;
        }

        Ok(())
    }


    pub async fn wait(&self) -> Result<TeamStatus> {
        let mut watch = self.status_watch();
        if watch.wait_for(|s| s.is_finished()).await.is_err() {
            return Err(Error::ChannelClosed { ch_name: "TeamStatus" });
        }

        let status = watch.borrow().clone();
        Ok(status)
    }

    pub async fn shutdown(&mut self) {
        self.shutdown_players().await;
        self.status_tx.send(TeamStatus::Idle).ok();
    }

    async fn shutdown_players(&mut self) {
        for mut player in &mut self.players.iter_mut() {
            let _ = player.value_mut().shutdown().await;
        }
        self.players.clear();
        self.agents.clear();
    }

    pub fn info(&self) -> TeamInfo {
        TeamInfo {
            name: self.config.name().to_string(),
            side: self.config.side(),
            status: self.status_now(),
            players: self.players.iter().map(|entry| (*entry.key(), entry.info())).collect(),
        }
    }
    
    pub fn len(&self) -> usize {
        self.players.len()
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Team is not finished from previous run.")]
    NotFinished,
    
    #[error("Channel {ch_name} already closed.")]
    ChannelClosed { ch_name: &'static str },
    
    #[error("Image '{image:?}' for policy is not found in registry.")]
    PolicyNotFound { image: ImageDeclaration },

    #[error("No matched metadata has been provided.")]
    NoMatchMetaData,

    #[error("Failed to spawn player: {0}")]
    SpawnPlayer(String),
}

pub type Result<T> = std::result::Result<T, Error>;
