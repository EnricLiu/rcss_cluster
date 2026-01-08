use tokio::sync::watch;
use common::command::trainer::{self, TrainerCommand};
use process::{CoachedProcessSpawner, CommandCaller};
use super::{BaseService, StandaloneArgs};

#[derive(Debug)]
pub struct StandaloneService {
    service: BaseService,
}

impl StandaloneService {
    pub async fn from_args(args: StandaloneArgs) -> crate::Result<Self> {
        let base = BaseService::from_args(args.base_args).await;
        Ok(Self::new(base))
    }

    fn new(service: BaseService) -> Self {
        StandaloneService { service }
    }

    pub fn shutdown_signal(&self) -> impl Future<Output=()> + 'static {
        // TODO!
        async {
            futures::future::pending::<()>().await;
        }
    }

    pub async fn spawn(&self) -> crate::Result<()> {
        self.service.spawn(true).await
    }

    pub async fn restart(&self, force: bool) -> crate::Result<()> {
        self.service.spawn(force).await
    }
}

impl std::ops::Deref for StandaloneService {
    type Target = BaseService;

    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

impl std::ops::DerefMut for StandaloneService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.service
    }
}
