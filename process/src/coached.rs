use crate::client::CommandCaller;
use crate::process::{self, ServerProcess, ServerProcessSpawner};
use crate::trainer::{self, OfflineCoach};
use std::time::Duration;
use common::command::trainer::TrainerCommand;
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

    pub async fn spawn(&self) -> Result<CoachedProcess, Box<dyn std::error::Error>> {
        let process = {
            let mut process = self.process.spawn().await?;
            match process.until_ready(Some(Duration::from_secs(2))).await {
                Ok(()) => {}
                Err(process::Error::TimeoutWaitingReady) => todo!("into"),
                Err(e) => {
                    panic!("{}", e);
                    todo!("fatal")
                }
            }
            process
        };

        let coach = {
            let coach = self.coach.build();
            coach.connect().await?;
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

    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.coach.shutdown().await?;
        self.process.shutdown().await?;
        Ok(())
    }

    pub fn command_sender(&self) -> CommandCaller<TrainerCommand> {
        self.coach().command_sender()
    }

    pub fn coach(&self) -> &OfflineCoach {
        &self.coach
    }
}
