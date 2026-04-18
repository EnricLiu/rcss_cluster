use kube::api::{Api, DeleteParams, ObjectList, PostParams};
use serde_json::Value;

use crate::metadata::{Labels, MetaData};
use crate::schema::v1::ConfigV1;

use super::builder::FleetBuilder;
use super::crd::Fleet;
use super::{Error, K8sClient, Result};


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
        let metadata: MetaData = gs_conf.try_into()
            .map_err(|e: common::errors::BuilderError| Error::InvalidMetaData(format!("{e:?}")))?;
        
        self.create_fleet_by_meta(name, metadata).await
    }

    pub async fn create_fleet_by_meta(&self, name: String, meta: MetaData) -> Result<()> {
        let api: Api<Fleet> = Api::namespaced(self.client.clone(), &self.agones_ns);

        let labels = meta.labels.try_into_map()
            .map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?;
        let annotations = meta.annotations.into_map();

        let fleet = {
            let mut fleet = FleetBuilder::new();
            fleet
                .with_name(name)
                .with_labels(labels)
                .with_annotations(annotations);

            fleet.build_into()
                .map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?
        };

        api.create(&PostParams::default(), &fleet).await
            .map_err(Error::CreateFleet)?;

        Ok(())
    }
    
    pub async fn drop_fleet(&self, name: &str) -> Result<()> {
        let api: Api<Fleet> = Api::namespaced(self.client.clone(), &self.agones_ns);

        match api.delete(name, &DeleteParams::default()).await {
            Ok(_) => Ok(()),
            Err(kube::Error::Api(err)) if err.code == 404 => Ok(()),
            Err(e) => Err(Error::DeleteFleet(e)),
        }
    }

    pub async fn fleet_by_labels(&self, labels: &Labels, limit: u32) -> Result<ObjectList<Fleet>> {
        let api: Api<Fleet> = Api::namespaced(self.client.clone(), &self.agones_ns);

        let labels = labels.try_as_map()
            .map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?;
        
        let selector = labels.into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>().join(",");

        let lp = kube::api::ListParams::default().labels(&selector).limit(limit);
        api.list(&lp).await.map_err(Error::SelectFleet)
    }
    
    pub async fn fleet_exists_by_labels(&self, labels: &Labels) -> Result<bool> {
        let fleets = self.fleet_by_labels(labels, 1).await?;
        Ok(!fleets.items.is_empty())
    }
}