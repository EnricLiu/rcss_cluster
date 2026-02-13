use tokio::sync::{watch, RwLock};
use log::{debug, info, warn};
use tokio::task::JoinHandle;
use common::command::{trainer, Command, CommandResult};
use common::command::trainer::TrainerCommand;
use process::{CoachedProcessSpawner, CommandCaller, ProcessConfig, ProcessStatus};

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

    cancel_tx: watch::Sender<bool>,
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
        let (cancel_tx, _) = watch::channel(false);
        Self { config, spawner, process, status_tx, status_rx, cancel_tx }
    }

    pub(crate) async fn spawn(&self, force: bool) -> Result<JoinHandle<()>> {
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

        let cancel_tx = self.cancel_tx.clone();
        let mut tasks: Vec<JoinHandle<()>> = vec![];

        let time_rx = process.time_watch();
        let status_tracing = tokio::spawn(
            Self::status_tracing_task(self.status_tx.clone(), time_rx, cancel_tx.clone())
        );
        tasks.push(status_tracing);
        info!("[BaseService] Status tracing task spawned");

        if let Some(half_time) = self.config.half_time_auto_start {
            let caller = process.trainer_command_sender();
            let kick_off_half_time = tokio::spawn(Self::kick_off_half_time_task(
                process.time_watch(),
                caller,
                half_time,
                cancel_tx.clone()
            ));
            tasks.push(kick_off_half_time);
            info!("[BaseService] KickOff Half-Time task spawned (half_time = {}ts)", half_time);
        }

        if self.config.always_log_stdout {
            let watcher = process.process_status_watch();
            let stdout_err_logging_task = tokio::spawn(Self::stdout_err_logging_task(
                watcher,
                cancel_tx.clone()
            ));
            tasks.push(stdout_err_logging_task);
        }

        *process_guard = OptionedProcess::Running(process);
        self.set_status(ServerStatus::Idle)
            .ok_or(Error::StatusChannelClosed)?;

        let ret = tokio::spawn(async move {
            let _ = futures::future::join_all(tasks).await;
        });

        Ok(ret)
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
        let _ = self.cancel_tx.send(true);

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
        mut time_rx: watch::Receiver<Option<u16>>,
        cancel_tx: watch::Sender<bool>,
    ) {
        let status_rx = status_tx.subscribe();
        let mut cancel_rx = cancel_tx.subscribe();
        loop {
            tokio::select! {
                res = time_rx.changed() => {
                    let timestep = match res {
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

                    if get_status(&status_rx).is_finished() {
                        info!("[BaseService] Status Tracking ended: server status finished.");
                        break;
                    };
                },

                _ = cancel_rx.changed() => {
                    info!("[BaseService] Status Tracking ended: cancel recved.");
                    break;
                },
            }
        }

        let _ = cancel_tx.send(true);
        info!("[BaseService] Status Tracking finished.");
    }

    /// trying to send start when half-time reached
    async fn kick_off_half_time_task(
        mut time_rx: watch::Receiver<Option<u16>>,
        caller: CommandCaller<TrainerCommand>,
        half_time: u16,
        cancel_tx: watch::Sender<bool>,
    ) {
        let mut cancel_rx = cancel_tx.subscribe();

        assert!(half_time > 0 && half_time < 6000,
            "[BaseService] kick_off_half_time_task: half_time must be between 1 and 5999");

        loop {
            tokio::select! {
                _ = cancel_rx.changed() => {
                    info!("[BaseService] KickOff Halftime ended: cancel recved.");
                },
                res = time_rx.changed() => {
                    if let Err(e) = res {
                        info!("[BaseService] KickOff Halftime ended: time_rx channel closed.");
                        let _ = cancel_tx.send(true);
                        break;
                    }

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

                },
            }
        }

        info!("[BaseService] KickOff Halftime finished.");
    }

    async fn stdout_err_logging_task(
        mut status: watch::Receiver<ProcessStatus>,
        cancel_tx: watch::Sender<bool>,
    ) {
        let mut cancel_rx = cancel_tx.subscribe();

        tokio::select! {
            res = status.wait_for(|s| s.is_finished()) => {
                if let Err(e) = &res {
                    warn!("[BaseService] Failed to wait for process status: {:?}", e);
                }
                info!("[BaseService] stdout_err_logging ended: process finished.");
                let _ = cancel_tx.send(true);
            },

            _ = cancel_rx.changed() => {
                info!("[BaseService] stdout_err_logging ended: cancel recved.");
            },
        }

        let status = status.borrow().clone();
        for (idx, line) in status.stdout_logs().await.into_iter().enumerate() {
            info!("[BaseService] Stdout {idx}: {line}");
        }
        for (idx, line) in status.stderr_logs().await.into_iter().enumerate() {
            warn!("[BaseService] Stderr {idx}: {line}");
        }

        info!("[BaseService] Stdout/stderr Logging: finished.")
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