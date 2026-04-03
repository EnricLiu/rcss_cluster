use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;

use log::warn;
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task::JoinHandle;
use dashmap::{DashMap, DashSet};
use tokio::pin;
use tokio::sync::watch::Ref;
use common::process::{ProcessStatus, ProcessStatusKind};
use crate::model::TeamModel;
use crate::player::{Player, PolicyPlayer};
use crate::policy::PolicyRegistry;
use crate::declarations::{ImageDeclaration, Unum};
use crate::info::{PlayerInfo, TeamInfo, TeamStatusInfo};
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

    monitor_task: Option<JoinHandle<()>>,
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
            monitor_task: None,
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

        // Start the aggregation task: listen for player events and drive TeamStatus.
        let monitor_task = {
            let player_watches = {
                self.players.iter()
                    .map(|p| (p.key(), p.status_watch().expect("The player process is initialized by the player.spawn().await, so the unwrap here should be safe.")))
                    .collect()
            };
            Self::spawn_monitor_task(
                &self.config, player_watches, self.status_tx.clone()
            )
        }?;
        self.monitor_task = Some(monitor_task);

        if let Err(_e) = self.status_tx.send(TeamStatus::Running) {
            self.shutdown().await;
            return Err(Error::ChannelClosed { ch_name: "TeamStatus" });
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
        // Abort the aggregation task first so it won't react to player shutdowns.
        if let Some(task) = self.monitor_task.take() {
            task.abort();
        }
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

    fn spawn_monitor_task(
        config: &TeamModel,
        status_watches: DashMap<Unum, watch::Receiver<ProcessStatus>>,
        status_tx: watch::Sender<TeamStatus>
    ) -> Result<JoinHandle<()>> {
        let team_name = config.name().to_string();

        let handle = tokio::spawn(async move {
            let n_players = status_watches.len();

            let status = std::sync::Mutex::new(status_tx.subscribe().borrow().clone());
            let snapshots: DashMap<Unum, _> = DashMap::new();

            let _snapshots = &snapshots;
            let _status = &status;
            let parse_status_update = || {
                let guard = _status.lock().expect("todo");
                match guard {
                    TeamStatus::Idle => {
                        let has_booting = snapshots.iter().any(|status| status.is_booting());
                        let next = TeamStatus::Starting;
                        *guard = next.clone();
                        has_booting.then_some(next)
                    },
                    TeamStatus::Starting => {
                        let all_running = snapshots.iter().all(|status| status.is_running());
                        let next = TeamStatus::Running;
                        *guard = next.clone();
                        all_running.then_some(next)
                    }
                }
            }

            let mut futures = Vec::with_capacity(status_watches.len());
            for (unum, mut watch) in status_watches {
                let fut = async {
                    loop {
                        if let Err(_) = watch.changed().await {
                            break Err(Error::ChannelClosed { ch_name: "player_process_status" });
                        }

                        let status = watch.borrow().status();
                        if let Some(status) = status.err() {
                            break Ok(TeamStatus::Error(Error::PlayerExited { unum, status, }))
                        }

                        let _last = snapshots.insert(unum, status);
                        if let Some(ret) = parse_status_update() {
                            break Ok(ret)
                        }
                    }
                };
                futures.push(pin!(fut))
            }

            futures::future::join_all(futures).await;

            // while let Some(event) = event_rx.recv().await {
            //     let err = Error::PlayerExited {
            //         unum: event.unum,
            //         status: format!("{:?}", event.status.status()),
            //     };
            //     warn!("[Team {team_name}] {err}");
            //     // Transition team to error — this unblocks Team::wait().
            //     let _ = status_tx.send(TeamStatus::Error(err));
            //     // After the first fatal event we stop listening; the team is
            //     // considered failed and will be shut down by the upper layer.
            //     return;
            // }
        });

        Ok(handle)
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

    #[error("Player {unum} exited unexpectedly: {status}")]
    PlayerExited { unum: Unum, status: String },
}

pub type Result<T> = std::result::Result<T, Error>;
