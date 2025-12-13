use super::CoachedProcess;
use super::addons;
use common::command::trainer::TrainerCommand;
use common::command::{Command, CommandResult};
use log::info;
use tokio::sync::watch;

#[derive(Debug)]
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
        let time_rx = process
            .coach()
            .add_caller_addon::<addons::TimeStatusAddon>("time");
        info!("[Service] Time status addon registered");

        Self { process, time_rx }
    }

    pub async fn send_trainer_command<C: Command<Kind = TrainerCommand>>(
        &self,
        command: C,
    ) -> CommandResult<C> {
        self.process.coach().call(command).await.unwrap()
    }

    pub fn time_watch(&self) -> watch::Receiver<Option<u16>> {
        self.time_rx.clone()
    }

    pub fn time(&self) -> Option<u16> {
        *self.time_rx.borrow()
    }

    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.process.shutdown().await?;
        Ok(())
    }
}
