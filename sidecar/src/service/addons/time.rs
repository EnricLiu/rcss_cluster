use std::time::Duration;

use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use log::debug;

use common::client::{RxData, TxData, TxSignal};
use common::command;
use common::command::trainer::{TrainerCommand};
use crate::client::{Addon, CallerAddon, CallSender};

#[derive(Debug)]
pub struct TimeStatusAddon<const POLL_INT_MS: u64 = 2000> {
    timestep: watch::Receiver<Option<u16>>,
    task: JoinHandle<()>,
}

impl<const POLL_INT_MS: u64> TimeStatusAddon<POLL_INT_MS> {
    fn start(caller: CallSender<TrainerCommand, TxData, RxData>) -> Self {
        let (time_tx, time_rx) = watch::channel(None);
        let task = tokio::spawn(async move {
            loop {
                if let Ok(Ok(res)) = caller.call(command::trainer::CheckBall).await {
                    time_tx.send(Some(res.time)).expect("Channel Closed"); // TODO: Handle error
                } else {
                    debug!("[TimeStatusAddon] Failed to get time: Caller closed.");
                    break;
                }
                tokio::time::sleep(Duration::from_millis(POLL_INT_MS)).await;
            }
        });

        Self {
            timestep: time_rx,
            task,
        }
    }

    fn watcher(&self) -> watch::Receiver<Option<u16>> {
        self.timestep.clone()
    }

    fn time(&self) -> Option<u16> {
        self.timestep.borrow().clone()
    }
}

impl<const POLL_INT_MS: u64> Addon for TimeStatusAddon<POLL_INT_MS> {
    fn close(&self) {
        self.task.abort()
    }
}

impl<const POLL_INT_MS: u64> CallerAddon<TrainerCommand> for TimeStatusAddon<POLL_INT_MS> {
    type Handle = watch::Receiver<Option<u16>>;
    
    fn handle(&self) -> Self::Handle {
        self.watcher()
    }
    
    fn from_caller(
        _: mpsc::Sender<TxSignal>,
        caller: CallSender<TrainerCommand, TxData, RxData>
    ) -> Self {
        Self::start(caller)
    }
}

#[cfg(test)]
mod tests {
    use crate::service;
    use super::*;

    #[tokio::test]
    async fn test_tracking_time_status_auto_start_half_time_break_end() -> Result<(),()> {
        let spawner = service::CoachedProcess::spawner().await;
        let server = spawner.spawn().await.expect("Spawn failed");

        let rx = server.coach().add_caller_addon::<TimeStatusAddon>("time");
        let caller = server.coach().caller();
        caller.call(command::trainer::Start).await.expect("Start failed").expect("Failed to start");

        let time_task = tokio::spawn(async move {
            let mut rx = rx;
            while let Ok(_) = rx.changed().await {
                let t = *rx.borrow();
                println!("Timestep:\t{t:?}.");
                if let Some(t) = t {
                    if t == 3000 {
                        caller.call(command::trainer::Start).await.expect("Start failed")
                            .expect("Failed to start at half-time");
                        continue
                    }
                    if t >= 6000 {
                        println!("Reached end of test at timestep {t}.");
                        break
                    }
                }
            }
        });

        tokio::time::timeout(Duration::from_secs(10), time_task)
            .await.expect("Timeout waiting for timestep >= 6000").unwrap();

        server.shutdown().await.expect("Shutdown failed");

        Ok(())
    }
}