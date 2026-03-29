use kube::Client;

mod fleet;
mod allocation;
pub mod error;

pub use error::{Error, Result};

pub use allocation::GsAllocation;


#[derive(Clone)]
pub struct K8sClient {
    client: Client,
}

impl K8sClient {
    pub async fn new() -> Result<Self> {
        let client = Client::try_default().await
            .map_err(Error::CreateClient)?;
        
        Ok(Self { client })
    }

    fn client(&self) -> &Client {
        &self.client
    }
}
