use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use futures::AsyncReadExt;
use log::{debug, info};
use tokio::sync::{watch, RwLock};
use tokio::task::JoinHandle;
use common::command::{Command, CommandResult};
use common::command::trainer::TrainerCommand;

use crate::{Error, Result};
use crate::process;
use crate::service::{Service, CoachedProcess, CoachedProcessSpawner};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum SidecarStatus {
    Uninitialized,
    Idle,
    Simulating,
    Finished,
}

impl SidecarStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, SidecarStatus::Simulating)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, SidecarStatus::Idle)
    }
}

impl Into<u8> for SidecarStatus {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for SidecarStatus {
    type Error = ();
    
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(SidecarStatus::Uninitialized),
            1 => Ok(SidecarStatus::Idle),
            2 => Ok(SidecarStatus::Simulating),
            3 => Ok(SidecarStatus::Finished),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum SidecarService {
    Uninitialized,
    Running(Service),
}

pub fn set_status(atomic: &AtomicU8, status: SidecarStatus) {
    atomic.store(status.into(), std::sync::atomic::Ordering::SeqCst);
}

pub fn get_status(atomic: &AtomicU8) -> SidecarStatus {
    SidecarStatus::try_from(atomic.load(std::sync::atomic::Ordering::SeqCst)).unwrap()
}

impl SidecarService {
    pub fn service(&self) -> Option<&Service> {
        match self {
            SidecarService::Uninitialized => None,
            SidecarService::Running(s) => Some(s),
        }
    }

    pub fn service_mut(&mut self) -> Option<&mut Service> {
        match self {
            SidecarService::Uninitialized => None,
            SidecarService::Running(s) => Some(s),
        }
    }

    pub async fn send_trainer_command<C>(&self, command: C) -> Option<CommandResult<C>>
    where C: Command<Kind=TrainerCommand>,
    {
        Some(self.service()?.send_trainer_command(command).await)
    }
    
    pub fn is_initialized(&self) -> bool {
        !matches!(self, SidecarService::Uninitialized)
    }

    pub fn is_running(&self) -> bool {
        matches!(self, SidecarService::Running(_))
    }
}

#[derive(Debug)]
pub struct Sidecar {
    spawner: CoachedProcessSpawner,
    service: RwLock<SidecarService>,
    status: Arc<AtomicU8>,
}

impl Sidecar {
    pub async fn new() -> Self {
        let spawner = CoachedProcess::spawner().await;
        let service = RwLock::new(SidecarService::Uninitialized);
        let status = Arc::new(AtomicU8::new(SidecarStatus::Uninitialized.into()));

        Self {
            spawner,
            service,
            status,
        }
    }
    async fn status_tracing_task(
        atomic: Arc<AtomicU8>,
        mut time_rx: watch::Receiver<Option<u16>>
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                let timestep = match time_rx.changed().await {
                    Ok(_) => *time_rx.borrow(),
                    Err(_) => {
                        set_status(&atomic, SidecarStatus::Finished);
                        debug!("[Sidecar] Status Tracking ended due to time_rx channel closed.");
                        break;
                    }
                };

                let next_status = match (get_status(&atomic), timestep) {
                    (SidecarStatus::Uninitialized, Some(0)) => SidecarStatus::Idle,
                    (SidecarStatus::Uninitialized, Some(_)) => SidecarStatus::Simulating,
                    (SidecarStatus::Idle, Some(t)) if t > 0 && t < 6000 => SidecarStatus::Simulating,
                    (SidecarStatus::Idle, Some(t)) if t >= 6000 => SidecarStatus::Finished,
                    (SidecarStatus::Simulating, Some(t)) if t >= 6000 => SidecarStatus::Finished,
                    _ => continue,
                };

                debug!("[Sidecar] Status Tracking: {:?} -> {:?}", get_status(&atomic), next_status);
                set_status(&atomic, next_status);
            }
        })
    }
    
    fn set_status(&self, status: SidecarStatus) {
        set_status(&self.status, status);
    }
    
    pub fn status(&self) -> SidecarStatus {
        get_status(&self.status)
    }

    pub async fn send_trainer_command<C: Command<Kind=TrainerCommand>>(&self, command: C) -> Result<CommandResult<C>> {
        self.service.read().await.send_trainer_command(command).await.ok_or(Error::ServerNotRunning {
            status: self.status()
        })
    }

    pub async fn spawn(&self) -> Result<()> {
        // >- service WRITE lock -<
        let mut service_guard = self.service.write().await;

        if let Some(service) = service_guard.service_mut() { // is running
            service.shutdown().await.expect("JB 没关掉");
        }
        let process = self.spawner.spawn().await.expect("JB 没开起来");
        let service = Service::from_coached_process(process);
        info!("[Sidecar] Service spawned");

        let time_rx = service.time_watch();
        Sidecar::status_tracing_task(self.status.clone(), time_rx);

        *service_guard = SidecarService::Running(service);
        self.set_status(SidecarStatus::Idle);

        Ok(())
        // >- service WRITE free -<
    }

    #[cfg(feature = "restart")]
    pub async fn restart(&self, force: bool) -> Result<()> {
        if force || !self.status().is_running() {
            return self.spawn().await;
        }

        Err(Error::ServerStillRunningToRestart)
    }
    
    pub fn config(&self) -> &process::Config {
        &self.spawner.process.config
    }
}
