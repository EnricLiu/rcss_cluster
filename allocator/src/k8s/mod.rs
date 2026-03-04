use kube::Client;

pub mod allocation;

#[derive(Clone)]
pub struct K8sClient {
    client: Client,
}

impl K8sClient {
    pub async fn new() -> Result<Self, kube::Error> {
        let client = Client::try_default().await?;
        Ok(Self { client })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
