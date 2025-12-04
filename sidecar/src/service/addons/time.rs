use std::time::Duration;

use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use log::debug;

use common::client::TxSignal;

use crate::coach::command;
use crate::coach::{Addon, CallerAddon, CallerSender};

pub const POLL_DURATION: Duration = Duration::from_millis(2000);

#[derive(Debug)]
pub struct TimeStatusAddon {
    timestep: watch::Receiver<Option<u16>>,
    task: JoinHandle<()>,
}

impl TimeStatusAddon {
    fn start(caller: CallerSender, duration: Duration) -> Self {
        let (time_tx, time_rx) = watch::channel(None);
        let task = tokio::spawn(async move {
            loop {
                if let Ok(Ok((time, _))) = caller.call(command::CheckBall).await {
                    time_tx.send(Some(time)).expect("Channel Closed"); // TODO: Handle error
                } else {
                    debug!("[TimeStatusAddon] Failed to get time: Caller closed.");
                    break;
                }
                tokio::time::sleep(duration).await;
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

impl Addon for TimeStatusAddon {
    fn close(&self) {
        self.task.abort()
    }
}

impl CallerAddon for TimeStatusAddon {
    type Handle = watch::Receiver<Option<u16>>;
    
    fn handle(&self) -> Self::Handle {
        self.watcher()
    }
    
    fn from_caller(_: mpsc::Sender<TxSignal>, caller: CallerSender) -> Self {
        Self::start(caller, POLL_DURATION)
    }
}