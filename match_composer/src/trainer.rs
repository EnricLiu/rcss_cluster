use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use tokio::sync::watch;

use common::process::ProcessStatus;

use crate::info::{TrainerInfo, TrainerStatusInfo};
use crate::model::TrainerBaseModel;
use crate::player::{PolicyProcess, Result};
use crate::policy::Policy;

pub type TrainerStatus = ProcessStatus;
pub type PolicyTrainer<Config> = PolicyProcess<Config>;

#[async_trait::async_trait]
pub trait Trainer: Debug + Send + Sync + 'static {
    fn model(&self) -> &TrainerBaseModel;
    fn status_watch(&self) -> Option<watch::Receiver<ProcessStatus>>;
    fn status_now(&self) -> Option<ProcessStatus> {
        self.status_watch().map(|w| w.borrow().clone())
    }
    async fn spawn(&self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
}

#[async_trait::async_trait]
impl<Config: Policy<Model = TrainerBaseModel> + Sync + Send + 'static> Trainer for PolicyTrainer<Config> {
    fn model(&self) -> &TrainerBaseModel {
        self.config.info()
    }

    fn status_watch(&self) -> Option<watch::Receiver<TrainerStatus>> {
        self.process.get().map(|p| p.status_watch())
    }

    async fn spawn(&self) -> Result<()> {
        self.spawn_process().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.shutdown_process().await
    }
}

#[derive(Debug)]
pub struct TrainerWrap(Box<dyn Trainer>);

impl<T: Trainer> From<T> for TrainerWrap {
    fn from(trainer: T) -> Self {
        Self(Box::new(trainer))
    }
}

impl Deref for TrainerWrap {
    type Target = Box<dyn Trainer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TrainerWrap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TrainerWrap {
    pub fn info(&self) -> TrainerInfo {
        let status = self.status_now()
            .map(|s| TrainerStatusInfo::Some(s.serialize()))
            .unwrap_or(TrainerStatusInfo::Unknown);

        let model = self.model();
        TrainerInfo {
            kind: model.kind,
            image: model.image.clone(),
            status,
        }
    }
}
