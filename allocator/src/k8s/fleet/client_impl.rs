use std::time::Duration;

use log::{debug, error, info, warn};
use kube::api::{DeleteParams, PostParams};
use serde_json::Value;

use common::errors::{BuilderError, BuilderResult};

use crate::k8s::crd::FleetStatus;
use crate::metadata::{Labels, MetaData};
use crate::schema::v1::ConfigV1;

use super::crd::Fleet;
use super::builder::FleetBuilder;
use super::{Error, K8sClient, Result};

#[derive(Debug, Clone)]
pub struct FleetName<'a> {
    pub raw: &'a str,
    pub name: &'a str,
    pub hash: &'a str,
}

impl<'a> FleetName<'a> {
    pub fn parse(raw: &'a str) -> BuilderResult<Self> {
        let mut parts = raw.rsplitn(2, '-');

        let res = match (parts.next(), parts.next()) {
            (Some(hash), Some(name)) => Self { raw, name, hash },
            _ => return Err(
                BuilderError::InvalidValue {
                    field: "fleet hash",
                    value: raw.to_string(),
                    expected: "a string in the format of <name>-<hash>".to_string(),
                }
            ),
        };

        Ok(res)
    }

    pub fn from_labels(labels: &'a Labels) -> Result<String> {
        let hash = labels.as_hash()
            .map_err(|e|
                Error::InvalidMetaData(
                    format!("Failed to compute hash for labels, error: {e:?}")
                )
            )?;

        Ok(Self::create_default(hash))
    }

    /// make sure no '-' in hash
    pub fn create(name: &str, hash: &str) -> String {
        format!("{name}-{hash}")
    }

    pub fn create_default(hash: &str) -> String {
        Self::create("fleet", hash)
    }

    pub fn hash(&self) -> &str {
        self.hash
    }

    pub fn name(&self) -> &str {
        self.name
    }
}


impl K8sClient {
    pub const DEFAULT_FLEET_READY_TIMEOUT: Duration = Duration::from_secs(100);

    pub async fn get_or_create_fleet(&self, gs_conf: Value, version: u8, timeout: Option<Duration>) -> Result<Fleet> {
        let meta: MetaData = match version {
            1 => {
                let conf: ConfigV1 = serde_json::from_value(gs_conf).map_err(Error::InvalidFleetGS)?;
                conf.try_into().map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?
            },
            _ => return Err(Error::UnsupportedVersion {
                version,
                resource: "Fleet",
                supported: &[1],
            }),
        };

        self.get_or_create_fleet_by_meta(meta, timeout).await
    }

    pub async fn get_or_create_fleet_by_meta(&self, meta: MetaData, timeout: Option<Duration>) -> Result<Fleet> {
        let fleet = self.fleet_by_labels(&meta.labels).await;
        let fleet_name = FleetName::from_labels(&meta.labels)?;

        match fleet {
            Ok(fleet) => {
                if let Some(ref st) = fleet.status && st.has_ready() {
                    return Ok(fleet);
                }
                info!("Fleet[{fleet_name}] already exists but not ready yet, will wait for it to be ready");
            },
            Err(Error::FleetNotFound) => {
                // fleet not found, try to create one
                match self.create_fleet_by_meta(&meta).await {
                    Ok(_) => debug!("Fleet[{fleet_name}] created successfully"),
                    Err(Error::FleetAlreadyExists { .. }) => debug!("Fleet[{fleet_name}] already exists"),
                    Err(e) => return Err(e),
                };
            },
            Err(e) => return Err(e),
        };

        self.fleet_poll_status(
            &meta.labels,
            FleetStatus::has_ready,
            None,
            timeout
        ).await
    }

    pub async fn drop_fleet(&self, name: &str) -> Result<()> {
        match self.fleet_client.delete(name, &DeleteParams::default()).await {
            Ok(_) => Ok(()),
            Err(kube::Error::Api(err)) if err.code == 404 => Ok(()),
            Err(e) => Err(Error::DeleteFleet(e)),
        }
    }

    /// no checking for label match
    pub async fn fleet_by_labels_unchecked(&self, labels: &Labels) -> Result<Fleet> {
        let fleet_name = FleetName::from_labels(labels)?;

        match self.fleet_client.get(&fleet_name).await {
            Ok(fleet) => Ok(fleet),
            Err(kube::Error::Api(err)) if err.code == 404 => Err(Error::FleetNotFound),
            Err(e) => Err(Error::SelectFleet(e)),
        }
    }

    pub async fn fleet_by_labels(&self, labels: &Labels) -> Result<Fleet> {
        let fleet = self.fleet_by_labels_unchecked(labels).await?;
        let fleet_name = fleet.metadata.name.clone()
            .ok_or_else(|| Error::InvalidMetaData("Fleet metadata.name is missing".to_string()))?;

        let matched_labels = match &fleet.metadata.labels {
            Some(labels) => labels,
            None => return Err(Error::FleetNotMatch {
                fleet: fleet_name,
                expected: format!("{labels:?}"),
                actual: "None".to_string(),
            }),
        };

        let true_labels = labels.try_as_ordered_map()
            .map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?;

        if !true_labels.eq(matched_labels) {
            error!("FATAL: [FleetByLabels] Fleet hash collision!");
            return Err(Error::FleetNotMatch {
                fleet: fleet_name,
                expected: format!("{true_labels:?}"),
                actual: format!("{matched_labels:?}"),
            });
        }

        Ok(fleet)
    }
    
    pub async fn fleet_exists_by_labels(&self, labels: &Labels) -> Result<bool> {
        match self.fleet_by_labels(labels).await {
            Ok(_) => Ok(true),
            Err(Error::FleetNotFound) | Err(Error::FleetNotMatch { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub async fn fleet_poll_status(
        &self,
        labels: &Labels,
        condition: impl Fn(&FleetStatus) -> bool,
        interval: Option<Duration>,
        timeout: Option<Duration>,
    ) -> Result<Fleet> {
        let fleet_name = FleetName::from_labels(labels)?;

        let fut = async {
            let mut interval = tokio::time::interval(interval.unwrap_or(self.retry_duration));
            loop {
                match self.fleet_by_labels_unchecked(labels).await {
                    Err(e) => warn!("Fleet[{fleet_name}] Failed to get while polling for ready, error: {e:?}"),
                    Ok(fleet) => {
                        if  let Fleet{ status: Some(ref status), .. } = fleet &&
                            condition(status)
                        {
                            info!("Fleet[{fleet_name}] is ready!");
                            return Ok(fleet);
                        } else {
                            debug!("Fleet[{fleet_name}] not ready yet, still waiting...")
                        }
                    },
                };
                interval.tick().await;
            }
        };

        tokio::select! {
            _ = tokio::time::sleep(timeout.unwrap_or(Self::DEFAULT_FLEET_READY_TIMEOUT)) => {
                Err(Error::FleetNotReady { fleet: fleet_name })
            },
            res = fut => res,
        }
    }

    async fn create_fleet_by_meta(&self, meta: &MetaData) -> Result<()> {
        let name = FleetName::from_labels(&meta.labels)?;

        let labels = meta.labels.try_as_map()
            .map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?;
        let annotations = meta.annotations.clone().into_map();


        let fleet = {
            let mut fleet = FleetBuilder::new();
            fleet
                .with_name(name)
                .with_labels(labels.clone())
                .with_annotations(annotations);

            fleet.build_into()
                .map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?
        };

        let fleet_name = fleet.metadata.name.as_deref().unwrap_or("<unknown>");

        match self.fleet_client.create(&PostParams::default(), &fleet).await {
            Ok(_) => {
                debug!("Fleet created. Fleet name: {fleet_name}",);
                Ok(())
            },
            Err(kube::Error::Api(err)) if err.code == 409 => {
                debug!("Fleet already exists. Fleet name: {fleet_name}");
                Err(Error::FleetAlreadyExists {
                    fleet: fleet_name.to_string(),
                })
            }
            Err(e) => Err(Error::CreateFleet {
                fleet: fleet_name.to_string(),
                source: e,
            }),
        }
    }
}
