use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use tokio::sync::watch;

use common::process::ProcessStatus;

use crate::info::{CoachInfo, CoachStatusInfo};
use crate::model::CoachBaseModel;
use crate::player::{PolicyProcess, Result};
use crate::policy::Policy;

pub type CoachStatus = ProcessStatus;
pub type PolicyCoach<Config> = PolicyProcess<Config>;

#[async_trait::async_trait]
pub trait Coach: Debug + Send + Sync + 'static {
    fn model(&self) -> &CoachBaseModel;
    fn status_watch(&self) -> Option<watch::Receiver<ProcessStatus>>;
    fn status_now(&self) -> Option<ProcessStatus> {
        self.status_watch().map(|w| w.borrow().clone())
    }
    async fn spawn(&self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
}

#[async_trait::async_trait]
impl<Config: Policy<Model = CoachBaseModel> + Sync + Send + 'static> Coach for PolicyCoach<Config> {
    fn model(&self) -> &CoachBaseModel {
        self.config.info()
    }

    fn status_watch(&self) -> Option<watch::Receiver<CoachStatus>> {
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
pub struct CoachWrap(Box<dyn Coach>);

impl<C: Coach> From<C> for CoachWrap {
    fn from(coach: C) -> Self {
        Self(Box::new(coach))
    }
}

impl Deref for CoachWrap {
    type Target = Box<dyn Coach>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CoachWrap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl CoachWrap {
    pub fn info(&self) -> CoachInfo {
        let status = self.status_now()
            .map(|s| CoachStatusInfo::Some(s.serialize()))
            .unwrap_or(CoachStatusInfo::Unknown);

        let model = self.model();
        CoachInfo {
            kind: model.kind,
            image: model.image.clone(),
            status,
        }
    }
}

