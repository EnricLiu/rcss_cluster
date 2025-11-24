use std::net::SocketAddr;
use std::time::Duration;
use crate::coach::{self, OfflineCoach};
use crate::process::{self, ServerProcess, ServerProcessSpawner};

use crate::RCSS_PROCESS_NAME;
use crate::PEER_IP;

pub struct ServerSpawner {
    coach:      coach::Builder,
    process:    ServerProcessSpawner,
}

impl ServerSpawner {
    pub async fn new() -> Self {
        ServerSpawner {
            coach: OfflineCoach::builder(),
            process: ServerProcess::spawner(RCSS_PROCESS_NAME).await,
        }
    }

    pub fn with_ports(
        &mut self,
        port: u16, coach_port: u16, olcoach_port: u16
    ) -> &mut Self {

        self.process.config_mut().with_ports(port, coach_port, olcoach_port);
        self.coach.with_peer(SocketAddr::new(PEER_IP, coach_port));

        self
    }

    pub async fn spawn(&self) -> Result<Server, Box<dyn std::error::Error>> {
        let process = {
            let mut process = self.process.spawn().await?;
            match process.until_ready(Some(Duration::from_secs(1))).await {
                Ok(()) => {}
                Err(process::Error::TimeoutWaitingReady) => todo!("into"),
                Err(_) => todo!("fatal"),
            }
            process
        };
        
        tokio::time::sleep(Duration::from_secs(1)).await; // todo!("remove this")
        
        let coach = {
            let mut coach = self.coach.build();
            coach.connect().await?;
            coach
        };
        
        Ok(Server::from_started(coach, process))
    }
}

#[derive(Debug)]
pub struct Server {
    coach:      OfflineCoach,
    process:    ServerProcess,
}

impl Server {
    fn from_started(coach: OfflineCoach, process: ServerProcess) -> Self {
        Server {
            coach,
            process,
        }
    }

    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        self.coach.shutdown().await?;
        self.process.shutdown().await?;
        Ok(())
    }
    
    pub fn coach(&self) -> &OfflineCoach {
        &self.coach
    }
}

