use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::process::ProcessStatus;
use common::types::Side;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::composer::AgentConnectionInfo;
use crate::config::{PlayerConfig, ServerConfig, TeamConfig};
use crate::image::ImageProcess;
use crate::policy::PolicyRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeamPhase {
    Init,
    Starting,
    Ready,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerKind {
    Bot,
    Agent,
}

#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub unum: u8,
    pub kind: PlayerKind,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone)]
pub struct TeamStatus {
    pub phase: TeamPhase,
    pub players: Vec<PlayerStatus>,
}

impl TeamStatus {
    pub fn is_finished(&self) -> bool {
        matches!(self.phase, TeamPhase::Ready | TeamPhase::Error(_))
    }
}

#[derive(Debug)]
pub struct Team {
    pub side: Side,
    pub config: TeamConfig,
    pub player_processes: Vec<ImageProcess>,
    pub agent_conns: Vec<AgentConnectionInfo>,

    status_tx: watch::Sender<TeamStatus>,
    status_state: Arc<Mutex<TeamStatus>>,
    monitor_tasks: Vec<JoinHandle<()>>,
}

impl Team {
    pub fn new(config: TeamConfig, side: Side) -> Self {
        let initial_status = TeamStatus {
            phase: TeamPhase::Init,
            players: Vec::new(),
        };
        let (status_tx, _) = watch::channel(initial_status.clone());
        Self {
            side,
            config,
            player_processes: Vec::new(),
            agent_conns: Vec::new(),
            status_tx,
            status_state: Arc::new(Mutex::new(initial_status)),
            monitor_tasks: Vec::new(),
        }
    }

    pub fn status_watch(&self) -> watch::Receiver<TeamStatus> {
        self.status_tx.subscribe()
    }

    pub async fn spawn(
        &mut self,
        registry: &PolicyRegistry,
    ) -> Result<(), String> {
        self.set_phase(TeamPhase::Starting);
        self.clear_players();
        let mut players = self.config.players.clone();

        players.sort_by_key(|p| p.unum());

        for player in players {
            let unum = player.unum();
            match player {
                PlayerConfig::Bot(bot_cfg) => {
                    let bot_policy = registry.fetch_bot(bot_cfg).ok_or_else(|| {
                        let err = "Failed to fetch bot policy".to_string();
                        self.set_phase(TeamPhase::Error(err.clone()));
                        err
                    })?;

                    self.player_processes.push(bot_policy.spawn().await);
                    self.push_player_status(unum, PlayerKind::Bot);
                }
                PlayerConfig::Agent(agent_cfg) => {
                    let grpc_cfg = agent_cfg.grpc.clone();
                    let agent_policy = registry.fetch_agent(agent_cfg).ok_or_else(|| {
                        let err = "Failed to fetch agent policy".to_string();
                        self.set_phase(TeamPhase::Error(err.clone()));
                        err
                    })?;

                    self.player_processes.push(agent_policy.spawn().await);
                    self.push_player_status(unum, PlayerKind::Agent);

                    self.agent_conns.push(AgentConnectionInfo {
                        side: self.side,
                        unum,
                        team_name: self.config.name.to_string(),
                        grpc_host: grpc_cfg.host,
                        grpc_port: grpc_cfg.port,
                    });
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        self.spawn_monitoring_tasks();
        self.set_phase(TeamPhase::Ready);

        Ok(())
    }

    pub async fn shutdown(&mut self) {
        for process in &mut self.player_processes {
            let _ = process.shutdown().await;
        }
        for task in self.monitor_tasks.drain(..) {
            task.abort();
        }
        self.player_processes.clear();
        self.agent_conns.clear();
        self.set_phase(TeamPhase::Init);
    }

    pub async fn wait(&self) -> Result<(), String> {
        let mut watch = self.status_watch();
        if watch.wait_for(|s| s.is_finished()).await.is_err() {
            return Err("Failed to watch team status".to_string());
        }

        let status = watch.borrow().clone();
        match status.phase {
            TeamPhase::Ready => Ok(()),
            TeamPhase::Error(err) => Err(err),
            _ => Err("Unexpected team status".to_string()),
        }
    }

    fn clear_players(&mut self) {
        let mut state = self.status_state.lock().unwrap();
        state.players.clear();
        let _ = self.status_tx.send(state.clone());
    }

    fn push_player_status(&mut self, unum: u8, kind: PlayerKind) {
        let status = self
            .player_processes
            .last()
            .map(|p| p.status_watch().borrow().clone())
            .unwrap_or(ProcessStatus::Init);

        let mut state = self.status_state.lock().unwrap();
        state.players.push(PlayerStatus { unum, kind, status });
        let _ = self.status_tx.send(state.clone());
    }

    fn set_phase(&self, phase: TeamPhase) {
        let mut state = self.status_state.lock().unwrap();
        state.phase = phase;
        let _ = self.status_tx.send(state.clone());
    }

    fn spawn_monitoring_tasks(&mut self) {
        if self.player_processes.is_empty() {
            return;
        }

        for (idx, process) in self.player_processes.iter().enumerate() {
            let mut watch = process.status_watch();
            let status_tx = self.status_tx.clone();
            let state = Arc::clone(&self.status_state);

            let handle = tokio::spawn(async move {
                loop {
                    if watch.changed().await.is_err() {
                        return;
                    }

                    let status = watch.borrow().clone();
                    let mut state_guard = state.lock().unwrap();
                    if let Some(player) = state_guard.players.get_mut(idx) {
                        player.status = status.clone();
                    }
                    if status.is_finished() {
                        let err_msg = format!(
                            "Player index {} process exited with status: {:?}",
                            idx, status
                        );
                        state_guard.phase = TeamPhase::Error(err_msg);
                    }
                    let _ = status_tx.send(state_guard.clone());

                    if status.is_finished() {
                        return;
                    }
                }
            });

            self.monitor_tasks.push(handle);
        }
    }
}
