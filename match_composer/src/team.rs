use std::collections::HashMap;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::time::Duration;

use log::{info, trace, warn};
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use dashmap::DashMap;
use allocator::schema::v1::CoachV1;
use common::process::{ProcessStatus, ProcessStatusKind};

use crate::coach::{Coach, CoachWrap, PolicyCoach};
use crate::model::TeamModel;
use crate::info::{PlayerInfo, PlayerStatusInfo, TeamInfo};
use crate::player::{Player, PolicyPlayer};
use crate::policy::PolicyRegistry;
use crate::declaration::{ImageDeclaration, Unum};

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
        let status = self.status_now()
            .map(|s| PlayerStatusInfo::Some(s.serialize()))
            .unwrap_or(PlayerStatusInfo::Unknown);

        let model = self.model();
        PlayerInfo {
            unum: model.unum,
            kind: model.kind,
            image: model.image.clone(),
            status,
        }
    }
}

#[derive(Debug)]
pub struct Team {
    pub config: TeamModel,

    status_tx: watch::Sender<TeamStatus>,
    status_rx: watch::Receiver<TeamStatus>,
    players: DashMap<Unum, PlayerWrap>,
    coach: Mutex<Option<CoachWrap>>,

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
            coach: Mutex::new(None),
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
        delay: Duration,
    ) -> Result<()> {
        if !self.status_tx.borrow().is_finished() {
            return Err(Error::NotFinished);
        }
        self.status_tx.send(TeamStatus::Starting)
            .map_err(|_| Error::ChannelClosed { ch_name: "TeamStatus" })?;

        self.ensure_log_dir().await;

        let mut players = self.config.players().clone()
            .into_iter().map(|(_, p)| p).collect::<Vec<_>>();

        players.sort_by_key(|p| p.unum);

        let mut interval = tokio::time::interval(delay);
        for player in players {
            let unum = player.unum;
            let policy = registry.fetch(player).map_err(|player| {
                let err = Error::PolicyNotFound { image: player.image.clone() };
                self.status_tx.send(TeamStatus::Error(err.clone())).ok();
                err
            })?;

            let player = PolicyPlayer::new(policy);
            player.spawn().await.map_err(|e|Error::SpawnPlayer(format!("{e:?}")))?;
            self.players.insert(unum, player.into());

            interval.tick().await;
        }


        if let Some(coach) = self.config.coach().cloned() {
            let policy = registry.fetch_coach(coach).map_err(|coach| {
                let err = Error::PolicyNotFound { image: coach.image.clone() };
                self.status_tx.send(TeamStatus::Error(err.clone())).ok();
                err
            })?;

            let coach = PolicyCoach::new(policy);
            coach.spawn().await.map_err(|e| Error::SpawnCoach(format!("{e:?}")))?;
            *self.coach.lock().await = Some(coach.into());
        }

        // Start the aggregation task: listen for player events and drive TeamStatus.
        let monitor_task = {
            let status_watches: HashMap<ParticipantId, watch::Receiver<ProcessStatus>> = {
                let mut watches: HashMap<ParticipantId, watch::Receiver<ProcessStatus>> = self.players.iter()
                    .map(|p| (
                        ParticipantId::Player(*p.key()),
                        p.status_watch().expect("The player process is initialized by the player.spawn().await, so the unwrap here should be safe.")
                    ))
                    .collect();

                if let Some(coach) = self.coach.lock().await.as_ref() {
                    watches.insert(
                        ParticipantId::Coach,
                        coach.status_watch().expect("The coach process is initialized by the coach.spawn().await, so the unwrap here should be safe."),
                    );
                }

                watches
            };
            Self::spawn_monitor_task(
                &self.config, status_watches, self.status_tx.clone()
            )
        }?;
        self.monitor_task = Some(monitor_task);


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
        self.shutdown_coach().await;
        self.shutdown_players().await;
        self.status_tx.send(TeamStatus::Idle).ok();
    }

    async fn shutdown_coach(&mut self) {
        let mut coach = self.coach.lock().await.take();
        if let Some(coach) = coach.as_mut() {
            let _ = coach.shutdown().await;
        }
    }

    async fn shutdown_players(&mut self) {
        for mut player in &mut self.players.iter_mut() {
            let _ = player.value_mut().shutdown().await;
        }
        self.players.clear();
    }

    async fn ensure_log_dir(&self) {
        if let Some(log_root) = &self.config.log_root {
            tokio::fs::create_dir_all(log_root).await
                .expect(&format!("Failed to create log directory {:?}", log_root));
        }
    }

    fn spawn_monitor_task(
        config: &TeamModel,
        status_watches: HashMap<ParticipantId, watch::Receiver<ProcessStatus>>,
        status_tx: watch::Sender<TeamStatus>
    ) -> Result<JoinHandle<()>> {
        let team_name = config.name().to_string();

        type WatchFut = Pin<Box<dyn
            Future<Output = (ParticipantId, Result<ProcessStatusKind>, watch::Receiver<ProcessStatus>)>
            + Send
        >>;

        fn next_change(id: ParticipantId, mut rx: watch::Receiver<ProcessStatus>) -> WatchFut {
            Box::pin(async move {
                let kind = match rx.changed().await {
                    Ok(()) => Ok(rx.borrow().kind.clone()),
                    Err(_) => Err(Error::ChannelClosed { ch_name: "ProcessStatus" }),
                };

                (id, kind, rx)
            })
        }

        let handle = tokio::spawn(async move {
            let mut snapshots: HashMap<ParticipantId, ProcessStatusKind> = {
                let mut map = HashMap::with_capacity(status_watches.len());
                for (id, rx) in status_watches.iter() {
                    map.insert(*id, rx.borrow().kind.clone());
                }
                map
            };

            let mut futs: FuturesUnordered<WatchFut> = status_watches.into_iter()
                .map(|(unum, rx)| next_change(unum, rx))
                .collect();

            while let Some((id, maybe_kind, rx)) = futs.next().await {
                let kind = match maybe_kind {
                    Ok(k) => k,
                    Err(e) => {
                        warn!("[{team_name}] {} status watch closed: {e}", id.label());
                        continue
                    }
                };

                trace!("[{team_name}] {} status: {}", id.label(), kind.name());
                snapshots.insert(id, kind);

                let new_status = Self::evaluate_team_status(&snapshots);
                let is_terminal = new_status.is_finished();

                status_tx.send_if_modified(|current| {
                    if current.kind() == new_status.kind() {
                        return false;
                    }
                    info!("[{team_name}] TeamStatus: {} -> {}", current.kind(), new_status.kind());
                    *current = new_status;
                    true
                });

                if is_terminal {
                    break;
                }

                futs.push(next_change(id, rx));
            }
        });

        Ok(handle)
    }

    fn evaluate_team_status(snapshots: &HashMap<ParticipantId, ProcessStatusKind>) -> TeamStatus {
        if snapshots.is_empty() {
            return TeamStatus::Starting;
        }

        let all_success  = snapshots.values().all(|s| s.is_success());
        if all_success { return TeamStatus::Idle; }

        let all_finished = snapshots.values().all(|s| s.is_finished());
        let first_err  = snapshots.iter().find(|(_, s)| s.is_err());
        if all_finished {
            if let Some((&id, kind)) = first_err {
                return TeamStatus::Error(Self::participant_exit_error(id, kind));
            }
        }

        if let Some((&id, kind)) = first_err {
            return TeamStatus::Aborting(Self::participant_exit_error(id, kind));
        }

        let any_success  = snapshots.iter().any(|(id, s)| {
            matches!(id, ParticipantId::Player(_)) && s.is_success()
        });
        if any_success {
            return TeamStatus::ShuttingDown;
        }

        let all_running  = snapshots.values().all(|s| s.is_running());
        if all_running { return TeamStatus::Running; }

        TeamStatus::Starting
    }

    fn participant_exit_error(id: ParticipantId, kind: &ProcessStatusKind) -> Error {
        let reason = kind.as_err().unwrap_or_default();
        match id {
            ParticipantId::Player(unum) => Error::PlayerExited { unum, reason },
            ParticipantId::Coach => Error::CoachExited { reason },
        }
    }

    pub fn info(&self) -> TeamInfo {
        let coach = {
            let coach = self.coach.try_lock().ok();
            if  let Some(coach) = &coach &&
                let Some(coach) = coach.as_ref() {
                Some(coach.info())
            } else {
                None
            }
        };

        TeamInfo {
            name: self.config.name().to_string(),
            side: self.config.side(),
            status: self.status_now().into(),
            players: self.players.iter().map(|entry| (*entry.key(), entry.info())).collect(),
            coach,
        }
    }
    
    pub fn len(&self) -> usize {
        self.players.len()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum ParticipantId {
    Player(Unum),
    Coach,
}

impl ParticipantId {
    fn label(self) -> String {
        match self {
            ParticipantId::Player(unum) => format!("Player {unum}"),
            ParticipantId::Coach => "Coach".to_string(),
        }
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

    #[error("Failed to spawn coach: {0}")]
    SpawnCoach(String),

    #[error("Player {unum} exited unexpectedly: {reason}")]
    PlayerExited { unum: Unum, reason: String },

    #[error("Coach exited unexpectedly: {reason}")]
    CoachExited { reason: String },
}

pub type Result<T> = std::result::Result<T, Error>;
