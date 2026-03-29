use serde_json::Value;
use crate::k8s::{Error, Result, K8sClient};
use crate::schema::v1::ConfigV1;

impl K8sClient {
    pub async fn create_fleet(&self, name: String, gs_conf: Value, version: u8) -> Result<()> {
        match version {
            1 => {
                let conf = serde_json::from_value(gs_conf).map_err(Error::InvalidFleetGS)?;
                self.create_fleet_v1(name, conf).await
            }
            _ => Err(Error::UnsupportedVersion {
                version,
                resource: "Fleet",
                supported: &[1],
            }),
        }
    }
    
    pub async fn create_fleet_v1(&self, name: String, gs_conf: ConfigV1) -> Result<()> {
        todo!()
    }
    
    pub async fn drop_fleet(&self, name: &str) -> Result<()> {
        todo!()
    }
}