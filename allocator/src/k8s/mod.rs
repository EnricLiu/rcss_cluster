use std::sync::Arc;
use std::time::Duration;
use kube::{Api, Client};
use arcstr::ArcStr;
use tokio::time::Interval;

mod fleet;
mod allocation;

pub mod error;

pub use error::{Error, Result};

pub use allocation::{
    GsAllocation,
    AllocationError,
    AllocationResult
};

pub use fleet::{
    init_fleet_template,
    fleet_template,
    fleet_template_version,
};

pub mod crd {
    pub use super::fleet::crd::*;
    pub use super::allocation::crd::*;
}


#[derive(Clone)]
pub struct K8sClient {
    agones_ns: ArcStr,
    client: Client,

    fleet_client: Arc<Api<crd::Fleet>>,
    alloc_client: Arc<Api<crd::GameServerAllocation>>,
    
    n_retry: usize,
    retry_duration: Duration,
}

impl K8sClient {
    pub async fn new(agones_ns: ArcStr, n_retry: usize, retry_duration: Duration) -> Result<Self> {
        let client = Client::try_default().await
            .map_err(Error::CreateClient)?;

        let fleet_client = Api::<crd::Fleet>::namespaced(client.clone(), &agones_ns);
        let alloc_client = Api::<crd::GameServerAllocation>::namespaced(client.clone(), &agones_ns);

        Ok(Self {
            agones_ns,
            client,
            n_retry,
            retry_duration,
            fleet_client: Arc::new(fleet_client),
            alloc_client: Arc::new(alloc_client),
        })
    }

    pub fn fleet_client(&self) -> &Api<crd::Fleet> {
        &self.fleet_client
    }

    pub fn alloc_client(&self) -> &Api<crd::GameServerAllocation> {
        &self.alloc_client
    }

    fn client(&self) -> &Client {
        &self.client
    }
    
    #[inline]
    pub fn n_retry_human(&self) -> usize {
        self.n_retry + 1
    }

    #[inline]
    pub fn retry_duration(&self) -> Duration {
        self.retry_duration
    }

    pub fn retry_interval(&self) -> Interval {
        tokio::time::interval(self.retry_duration())
    }
}
