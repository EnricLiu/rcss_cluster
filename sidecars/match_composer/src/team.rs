use crate::config::{ImageQuery, PlayerConfig, TeamConfig};
use crate::player::{Player, PlayerMeta, PolicyMeta, PolicyPlayer};
use crate::policy::PolicyRegistry;
use common::types::Side;
use dashmap::{DashMap, DashSet};
use serde::Serialize;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use tokio::sync::watch;

#[derive(Serialize, Clone, Debug)]
pub struct TeamMeta {
    pub name: String,
    pub side: Side,
    pub policies: Vec<PolicyMeta>,
}

#[derive(Debug, Clone)]
pub enum TeamStatus {
    Idle,
    Starting,
    Running,
    ShuttingDown,
    Error(Error),
}

impl TeamStatus {
    pub fn is_finished(&self) -> bool {
        matches!(self, TeamStatus::Idle | TeamStatus::Error(_))
    }
}

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

#[derive(Debug)]
pub struct Team {
    pub config: TeamConfig,

    status_tx: watch::Sender<TeamStatus>,
    status_rx: watch::Receiver<TeamStatus>,
    players: DashMap<u8, PlayerWrap>,
    agents: DashSet<u8>,
}

impl Team {
    pub fn new(config: TeamConfig) -> Self {
        let (status_tx, status_rx) = watch::channel(TeamStatus::Idle);
        Self {
            config,
            status_tx,
            status_rx,
            players: DashMap::new(),
            agents: DashSet::new(),
        }
    }
    
    pub fn meta(&self) -> TeamMeta {
        let policies = self.policies();
        TeamMeta {
            side: self.config.side,
            name: self.config.name.clone(),
            policies,
        }
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

        let mut players = self.config.players.clone();

        players.sort_by_key(|p| p.unum());

        for player in players {
            let unum = player.unum();
            let policy = registry.fetch(player).map_err(|player| {
                let err = {
                    let (image, player) = match player {
                        PlayerConfig::Bot(bot) => (bot.image.clone(), bot.into()),
                        PlayerConfig::Agent(agent) => (agent.image.clone(), agent.into()),
                    };

                    Error::PolicyNotFound { image, player }
                };
                self.status_tx.send(TeamStatus::Error(err.clone())).ok();
                err
            })?;

            if policy.meta().player.kind.is_agent() {
                if self.agents.contains(&unum) { continue }
                self.agents.insert(unum);
            }

            let player = PolicyPlayer::new(policy);
            player.spawn().await.map_err(|e|Error::SpawnPlayer(format!("{e:?}")))?;
            self.players.insert(unum, player.into());
            
            // match player {
            //     PlayerConfig::Bot(bot_cfg) => {
            //         let bot_policy = registry.fetch_bot(bot_cfg).ok_or_else(|| {
            //             let err = Error::PolicyNotFound { image: bot_cfg.image, player: bot_cfg.into() };
            //             self.status_tx.send(TeamStatus::Error(err)).ok();
            //             err
            //         })?;
            // 
            //         self.players.insert(unum, PolicyPlayer::new(bot_policy).into());
            //     }
            //     PlayerConfig::Agent(agent_cfg) => {
            //         let agent_policy = registry.fetch_agent(agent_cfg).ok_or_else(|| {
            //             let err = Error::PolicyNotFound { image: agent_cfg.image, player: agent_cfg.into() };
            //             self.status_tx.send(TeamStatus::Error(err)).ok();
            //             err
            //         })?;
            // 
            //         self.players.insert(unum, PolicyPlayer::new(agent_policy).into());
            //         self.agents.insert(unum);
            //     }
            // }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        if let Err(e) = self.status_tx.send(TeamStatus::Running) {
            self.shutdown().await;
        }

        Ok(())
    }


    pub async fn wait(&self) -> Result<()> {
        let mut watch = self.status_watch();
        if watch.wait_for(|s| s.is_finished()).await.is_err() {
            return Err(Error::ChannelClosed { ch_name: "TeamStatus" });
        }

        let status = watch.borrow().clone();
        match status {
            TeamStatus::Idle => Ok(()),
            TeamStatus::Error(err) => Err(err),
            _ => unreachable!("wait_for should only return when status is Idle or Error"),
        }
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

    pub fn policies(&self) -> Vec<PolicyMeta> {
        let mut policies = Vec::with_capacity(self.players.len());
        for player in self.players.iter() {
            policies.push(player.meta());
        }
        policies
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
    PolicyNotFound { image: ImageQuery, player: PlayerMeta },

    #[error("Failed to spawn player: {0}")]
    SpawnPlayer(String),
}

pub type Result<T> = std::result::Result<T, Error>;
