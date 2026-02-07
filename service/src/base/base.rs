use tokio::sync::{watch, RwLock};
use log::{debug, info, warn};

use common::command::{trainer, Command, CommandResult};
use common::command::trainer::TrainerCommand;
use process::{CoachedProcessSpawner, CommandCaller, ProcessConfig};

use crate::{Error, Result};
use super::{AddonProcess, BaseArgs, BaseConfig, ServerStatus};

#[derive(Debug)]
pub enum OptionedProcess {
    Uninitialized,
    Running(AddonProcess),
}
impl OptionedProcess {
    pub fn process(&self) -> Option<&AddonProcess> {
        match self {
            OptionedProcess::Uninitialized => None,
            OptionedProcess::Running(s) => Some(s),
        }
    }

    pub fn process_mut(&mut self) -> Option<&mut AddonProcess> {
        match self {
            OptionedProcess::Uninitialized => None,
            OptionedProcess::Running(s) => Some(s),
        }
    }

    pub async fn send_trainer_command<C>(&self, command: C) -> Option<CommandResult<C>>
    where C: Command<Kind = TrainerCommand>,
    {
        self.process()?.send_trainer_command(command).await.ok()
    }

    pub fn is_initialized(&self) -> bool {
        !matches!(self, OptionedProcess::Uninitialized)
    }

    pub fn is_running(&self) -> bool {
        matches!(self, OptionedProcess::Running(_))
    }
}


#[derive(Debug)]
pub struct BaseService {
    config: BaseConfig,
    spawner: CoachedProcessSpawner,
    process: RwLock<OptionedProcess>,
    status_tx: watch::Sender<ServerStatus>,
    status_rx: watch::Receiver<ServerStatus>,
}

#[must_use]
pub(crate) fn set_status(watch: &watch::Sender<ServerStatus>, status: ServerStatus) -> Option<()> {
    watch.send(status).ok()
}

pub(crate) fn get_status(watch: &watch::Receiver<ServerStatus>) -> ServerStatus {
    *watch.borrow()
}

impl BaseService {
    pub async fn from_args(args: BaseArgs) -> Self {
        let config = (&args).into();
        let mut spawner = CoachedProcessSpawner::new().await;
        let rcss_log_dir = args.rcss_log_dir.leak(); // STRING LEAK
        spawner
            .with_ports(args.player_port, args.trainer_port, args.coach_port)
            .with_sync_mode(args.rcss_sync)
            .with_log_dir(rcss_log_dir);

        BaseService::new(config, spawner).await
    }

    pub(super) async fn new(config: BaseConfig, spawner: CoachedProcessSpawner) -> Self {
        let process = RwLock::new(OptionedProcess::Uninitialized);
        let (status_tx, status_rx) = watch::channel(ServerStatus::Uninitialized);
        Self { config, spawner, process, status_tx, status_rx }
    }

    pub(crate) async fn spawn(&self, force: bool) -> Result<()> {
        // >- process WRITE lock -<
        let mut process_guard = self.process.write().await;

        if let Some(process) = process_guard.process_mut() {
            // is running
            if !force && self.status_now().is_running() {
                return Err(Error::ServerStillRunningToSpawn)
            }

            warn!("[BaseService] Force restarting the process...");
            if let Err(e) = process.shutdown().await {
                warn!("[BaseService] Failed to shutdown existing process: {:?}. dropping", e);
            }
        }
        self.set_status(ServerStatus::Uninitialized)
            .ok_or(Error::StatusChannelClosed)?;

        let process = self.spawner.spawn().await
            .map_err(|e| Error::ProcessSpawnFailed(e))?;
        let process = AddonProcess::from_coached_process(process);
        info!("[BaseService] AddonProcess spawned");

        let time_rx = process.time_watch();
        tokio::spawn(Self::status_tracing_task(self.status_tx.clone(), time_rx));
        info!("[BaseService] Status tracing task spawned");

        if let Some(half_time) = self.config.half_time_auto_start {
            let caller = process.trainer_command_sender();
            tokio::spawn(Self::kick_off_half_time_task(
                process.time_watch(),
                caller,
                half_time,
            ));
            info!("[BaseService] KickOff Half-Time task spawned (half_time = {}ts)", half_time);
        }

        *process_guard = OptionedProcess::Running(process);
        self.set_status(ServerStatus::Idle)
            .ok_or(Error::StatusChannelClosed)?;

        Ok(())
        // >- process WRITE free -<
    }

    pub async fn send_trainer_command<C>(&self, command: C) -> Result<CommandResult<C>>
    where C: Command<Kind = TrainerCommand> {
        self.process.read().await.process()
            .ok_or(Error::ServerNotRunning { status: ServerStatus::Uninitialized })?
            .send_trainer_command(command).await
            .map_err(|_| Error::Timeout { op: "send_trainer_command" })
    }

    pub async fn trainer_command_sender(&self) -> Result<CommandCaller<TrainerCommand>> {
        let ret = self.process.read().await.process()
            .ok_or(Error::ServerNotRunning { status: ServerStatus::Uninitialized })?
            .trainer_command_sender();
        Ok(ret)
    }

    pub async fn shutdown(&self) -> Result<()> {
        // >- process WRITE lock -<
        let mut process_guard = self.process.write().await;

        info!("[BaseService] Shutdown called, shutting down process if running...");

        if let Some(process) = process_guard.process_mut() {
            debug!("[BaseService] Shutting down existing process...");
            if let Err(e) = process.shutdown().await {
                warn!("[BaseService] Failed to shutdown existing process: {:?}.", e);
                return Err(Error::ProcessFailedToShutdown);
            }

            if self.set_status(ServerStatus::Shutdown).is_none() {
                warn!("[BaseService] Status channel closed during shutdown");
            }
        } else {
            debug!("[BaseService] Shutdown called but no process to shutdown.");
        }

        info!("[BaseService] Process shutdown complete.");

        Ok(())
        // >- process WRITE free -<
    }

    async fn status_tracing_task(
        status_tx: watch::Sender<ServerStatus>,
        mut time_rx: watch::Receiver<Option<u16>>
    ) {
        let status_rx = status_tx.subscribe();
        loop {
            let timestep = match time_rx.changed().await {
                Ok(_) => *time_rx.borrow(),
                Err(_) => {
                    let _ = set_status(&status_tx, ServerStatus::Finished);
                    info!("[BaseService] Status Tracking ended: time_rx channel closed.");
                    break;
                }
            };

            let next_status = match (get_status(&status_rx), timestep) {
                (ServerStatus::Uninitialized, Some(0)) => ServerStatus::Idle,
                (ServerStatus::Uninitialized, Some(_)) => ServerStatus::Simulating,
                (ServerStatus::Idle, Some(t)) if t > 0 && t < 6000 => {
                    ServerStatus::Simulating
                }
                (ServerStatus::Idle, Some(t)) if t >= 6000 => ServerStatus::Finished,
                (ServerStatus::Simulating, Some(t)) if t >= 6000 => ServerStatus::Finished,
                _ => continue,
            };

            debug!("[BaseService] Status Tracking: {:?} -> {:?}",
                get_status(&status_rx),next_status);

            if set_status(&status_tx, next_status).is_none() {
                info!("[BaseService] Status Tracking ended: status_tx channel closed.");
                break;
            }
        }
    }

    /// trying to send start when half-time reached
    async fn kick_off_half_time_task(
        mut time_rx: watch::Receiver<Option<u16>>,
        caller: CommandCaller<TrainerCommand>,
        half_time: u16,
    ) {
        assert!(half_time > 0 && half_time < 6000,
            "[BaseService] kick_off_half_time_task: half_time must be between 1 and 5999");

        while let Ok(_) = time_rx.changed().await {
            let time = match *time_rx.borrow() {
                Some(t) => t,
                None => continue,
            };

            // rcss server always stop at the half-time point
            // thus the equality check would be safe
            if time != half_time { continue };
            match caller.call(trainer::Start).await {
                Ok(_) =>
                    debug!("[BaseService] KickOff Halftime: Sent Start command at half-time {}", half_time),
                Err(e) =>
                    warn!("[BaseService] KickOff Halftime: Failed to send Start command at {}ts: {:?}", half_time, e),
            }
            break;
        }
    }

    #[must_use]
    pub(crate) fn set_status(&self, status: ServerStatus) -> Option<()> {
        set_status(&self.status_tx, status)
    }

    pub fn status_now(&self) -> ServerStatus {
        get_status(&self.status_rx)
    }

    pub fn status(&self) -> watch::Receiver<ServerStatus> {
        self.status_rx.clone()
    }

    pub async fn time_now(&self) -> Option<u16> {
        self.process.read().await.process().and_then(|p| p.time())
    }

    pub async fn time(&self) -> Option<watch::Receiver<Option<u16>>> {
        self.process.read().await.process().map(|p| p.time_watch())
    }

    pub fn config(&self) -> &ProcessConfig {
        &self.spawner.process.config
    }
}