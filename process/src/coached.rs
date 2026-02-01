use crate::process::{self, ServerProcess, ServerProcessSpawner};
use crate::trainer::{self, OfflineCoach};
use std::time::Duration;
use log::error;
use crate::{Error, Result};

use crate::RCSS_PROCESS_NAME;

#[derive(Clone, Debug)]
pub struct CoachedProcessSpawner {
    pub coach: trainer::Builder,
    pub process: ServerProcessSpawner,
}

impl CoachedProcessSpawner {
    pub async fn new() -> Self {
        CoachedProcessSpawner {
            coach: OfflineCoach::builder(),
            process: ServerProcess::spawner(RCSS_PROCESS_NAME).await,
        }
    }

    pub fn with_ports(&mut self, port: u16, coach_port: u16, olcoach_port: u16) -> &mut Self {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        self.process_config_mut()
            .with_ports(port, coach_port, olcoach_port);
        self.coach
            .with_peer(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), coach_port));

        self
    }
    
    pub fn with_sync_mode(&mut self, sync: bool) -> &mut Self {
        self.process_config_mut().with_sync(sync);
        self
    }
    
    pub fn with_log_dir(&mut self, log_dir: &'static str) -> &mut Self {
        self.process_config_mut().with_log_dir(log_dir);
        self
    }

    pub fn process_config_mut(&mut self) -> &mut process::Config {
        self.process.config_mut()
    }

    pub async fn spawn(&self) -> Result<CoachedProcess> {
        let process = {
            let mut process = self.process.spawn().await
                .map_err(|e| Error::SpawnProcess(e))?;
            let res = process.until_ready(Some(Duration::from_secs(2))).await;
            if res.is_err() {
                match &res {
                    Err(process::Error::TimeoutWaitingReady) => {
                        error!("CoachedProcessSpawner: process failed to become ready in time, killing process");
                        match process.shutdown().await {
                            Ok(exit_status) => {
                                error!("CoachedProcessSpawner: process killed, exit status: {}", exit_status);
                            },
                            Err(e) => {
                                error!("CoachedProcessSpawner: failed to kill process: {}", e);
                            },
                        }
                    },
                    Err(e) => {
                        error!("CoachedProcessSpawner: fatal error while waiting for process to become ready: {}", e);
                    }
                    Ok(()) => unreachable!("unreachable"),
                }

                let stdout_trace = process.stdout_logs().await;
                let stderr_trace = process.stderr_logs().await;

                error!("CoachedProcessSpawner: process stdout:\n{:?}", stdout_trace);
                error!("CoachedProcessSpawner: process stderr:\n{:?}", stderr_trace);

                return Err(Error::SpawnProcess(res.unwrap_err()))
            }
            process
        };

        let coach = {
            let coach = self.coach.build();
            coach.connect().await.map_err(|e| Error::ConnectCoach(e))?;
            coach
        };

        Ok(CoachedProcess::from_started(coach, process))
    }
}

#[derive(Debug)]
pub struct CoachedProcess {
    coach: OfflineCoach,
    process: ServerProcess,
}

impl CoachedProcess {
    pub async fn spawner() -> CoachedProcessSpawner {
        CoachedProcessSpawner::new().await
    }

    fn from_started(coach: OfflineCoach, process: ServerProcess) -> Self {
        CoachedProcess { coach, process }
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.coach.shutdown().await.map_err(|e| Error::ShutdownCoach(e))?;
        self.process.shutdown().await.map_err(|e| Error::ShutdownProcess(e))?;
        Ok(())
    }

    pub fn coach(&self) -> &OfflineCoach {
        &self.coach
    }
}
