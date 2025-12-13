pub mod addons;
mod coached;
mod service;

pub use coached::{CoachedProcess, CoachedProcessSpawner};
pub use service::Service;

use log::{info, trace, warn};

pub const GAME_END_TIMESTEP: u16 = 6000;

pub struct AgonesService {
    sdk: agones::Sdk,
    pub service: Service,
}

pub async fn start_singleton_service() -> Result<(), Box<dyn std::error::Error>> {
    let spawner = CoachedProcess::spawner().await;
    let config = spawner.process.config.clone();

    loop {
        let mut service = Service::from_coached_process(spawner.spawn().await?);
        info!("[Service] Spawned.");

        let mut time_watcher = service.time_watch();
        let restart_task = tokio::spawn(async move {
            let res = time_watcher
                .wait_for(|t| t.is_some_and(|t| t >= GAME_END_TIMESTEP))
                .await;
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
