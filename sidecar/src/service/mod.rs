mod coached;
mod addons;

pub use coached::CoachedProcess;
use coached::CoachedProcessSpawner;

use log::{info, trace, warn};
use tokio::sync::watch;

pub const GAME_END_TIMESTEP: u16 = 6000;

pub struct AgonesService {
    sdk: agones::Sdk,
    pub service: Service,
}

pub async fn start_singleton_service() -> Result<(), Box<dyn std::error::Error>> {
    let spawner = CoachedProcess::spawner().await;

    loop {
        let service = Service::from_coached_process(spawner.spawn().await?);
        info!("[Service] Spawned.");

        let mut time_watcher = service.time_watch();
        let restart_task = tokio::spawn(async move {
            let res = time_watcher.wait_for(|t|
                t.is_some_and(|t| t >= GAME_END_TIMESTEP)).await;
            trace!("[Service] Time watcher: {:?}", res);
            res.map(|_| ())
        });

        tokio::select! {
            res = restart_task => {
                info!("[Service] Restarting service");
                service.shutdown().await?;
                info!("[Service] shutdown: {res:?}");
            },

            _ = tokio::signal::ctrl_c() => {
                info!("[Service] Ctrl-C detected, Shutting Down");
                service.shutdown().await?;
                info!("[Service] shutdown, exiting.");
                break;
            }
        }
    }

    Ok(())
}

pub struct Service {
    process: CoachedProcess,
    time_rx: watch::Receiver<Option<u16>>,
}

impl Service {
    pub async fn new() -> Self {
        let spawner = CoachedProcess::spawner().await;
        let process = spawner.spawn().await.unwrap();
        info!("[Service] Process spawned");

        Self::from_coached_process(process)
    }

    pub fn from_coached_process(process: CoachedProcess) -> Self {
        let time_rx = process.coach()
            .add_caller_addon::<addons::TimeStatusAddon>("time");
        info!("[Service] Time status addon registered");

        Self {
            process,
            time_rx,
        }
    }

    pub fn time_watch(&self) -> watch::Receiver<Option<u16>> {
        self.time_rx.clone()
    }

    pub fn time(&self) -> Option<u16> {
        self.time_rx.borrow().clone()
    }

    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        self.process.shutdown().await?;
        Ok(())
    }
}
