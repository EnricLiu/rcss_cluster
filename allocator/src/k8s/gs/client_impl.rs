use std::time::Duration;

use kube::api::{DeleteParams, ListParams, Patch, PatchParams, PostParams};
use log::{debug, info, warn};
use serde_json::json;

use crate::metadata::MetaData;
use crate::utils::snowflake;

use super::builder::GameServerBuilder;
use super::crd::GameServer;
use super::lifecycle::{
    ALLOCATION_MODE_DIRECT_GS, ALLOCATION_MODE_LABEL, CREATED_AT_ANNOTATION,
    LAST_HEARTBEAT_ANNOTATION, MANAGED_BY_ALLOCATOR, MANAGED_BY_LABEL, TEMPLATE_VERSION_LABEL,
    managed_direct_selector, now_unix_secs,
};
use super::{Error, K8sClient, Result};

pub fn generate_gs_name() -> String {
    let version = super::gs_template_version();
    let id = snowflake::next_id();
    format!("gs-{version}-{id}")
}

impl K8sClient {
    pub const DEFAULT_GS_READY_TIMEOUT: Duration = Duration::from_secs(100);

    pub async fn create_gs_by_meta(&self, meta: &MetaData) -> Result<String> {
        let name = generate_gs_name();

        let labels = meta.labels.try_as_map()
            .map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?;
        let mut labels = labels.clone();
        labels.insert(
            MANAGED_BY_LABEL.to_string(),
            MANAGED_BY_ALLOCATOR.to_string(),
        );
        labels.insert(
            ALLOCATION_MODE_LABEL.to_string(),
            ALLOCATION_MODE_DIRECT_GS.to_string(),
        );
        labels.insert(
            TEMPLATE_VERSION_LABEL.to_string(),
            super::gs_template_version().to_string(),
        );

        let mut annotations = meta.annotations.clone().into_map();
        annotations.insert(
            CREATED_AT_ANNOTATION.to_string(),
            now_unix_secs().to_string(),
        );
        annotations.insert(
            LAST_HEARTBEAT_ANNOTATION.to_string(),
            now_unix_secs().to_string(),
        );

        let gs = {
            let mut gs = GameServerBuilder::new();
            gs.with_name(&name)
                .with_labels(labels)
                .with_annotations(annotations);

            gs.build_into()
                .map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?
        };

        match self.gs_client.create(&PostParams::default(), &gs).await {
            Ok(_) => {
                debug!("GameServer created. Name: {name}");
                Ok(name)
            }
            Err(kube::Error::Api(err)) if err.code == 409 => {
                debug!("GameServer already exists. Name: {name}");
                Err(Error::GsAlreadyExists { gs: name })
            }
            Err(e) => Err(Error::CreateGs {
                gs: name,
                source: e,
            }),
        }
    }

    pub async fn drop_gs(&self, name: &str) -> Result<()> {
        match self.gs_client.delete(name, &DeleteParams::default()).await {
            Ok(_) => Ok(()),
            Err(kube::Error::Api(err)) if err.code == 404 => Ok(()),
            Err(e) => Err(Error::DeleteGs(e)),
        }
    }

    pub async fn heartbeat_gs(&self, name: &str) -> Result<()> {
        let patch = json!({
            "metadata": {
                "annotations": {
                    LAST_HEARTBEAT_ANNOTATION: now_unix_secs().to_string(),
                }
            }
        });

        self.gs_client
            .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map(|_| ())
            .map_err(Error::SelectGs)
    }

    pub async fn list_managed_direct_gs(&self) -> Result<Vec<GameServer>> {
        let lp = ListParams::default().labels(&managed_direct_selector());
        self.gs_client
            .list(&lp)
            .await
            .map(|list| list.items)
            .map_err(Error::SelectGs)
    }

    pub async fn gs_poll_by_name(
        &self,
        name: &str,
        condition: impl Fn(&GameServer) -> bool,
        interval: Option<Duration>,
        timeout: Option<Duration>,
    ) -> Result<GameServer> {
        let gs_name = name.to_string();

        let fut = async {
            let mut interval = tokio::time::interval(interval.unwrap_or(self.retry_duration));
            loop {
                match self.gs_client.get(&gs_name).await {
                    Err(e) => {
                        warn!("GameServer[{gs_name}] Failed to get while polling, error: {e:?}")
                    }
                    Ok(gs) => {
                        if condition(&gs) {
                            info!("GameServer[{gs_name}] is ready!");
                            return Ok(gs);
                        } else {
                            debug!(
                                "GameServer[{gs_name}] not ready yet, state: {:?}",
                                gs.status.as_ref().map(|s| &s.state)
                            )
                        }
                    }
                };
                interval.tick().await;
            }
        };

        tokio::select! {
            _ = tokio::time::sleep(timeout.unwrap_or(Self::DEFAULT_GS_READY_TIMEOUT)) => {
                Err(Error::GsNotReady { gs: gs_name })
            },
            res = fut => res,
        }
    }

    pub async fn create_and_wait_gs_by_meta(
        &self,
        meta: MetaData,
        timeout: Option<Duration>,
    ) -> Result<GameServer> {
        let gs_name = loop {
            let res = self.create_gs_by_meta(&meta).await;
            match res {
                Err(Error::GsAlreadyExists { gs }) => {
                    warn!("GameServer[{gs}] already exists, retrying...")
                }
                Ok(gs) => break gs,
                Err(e) => return Err(e),
            }
        };

        let ready = self
            .gs_poll_by_name(
                &gs_name,
                |gs| gs.status.as_ref().map_or(false, |s| s.is_ready()),
                None,
                timeout,
            )
            .await
            .inspect_err(|err| {
                warn!("GameServer[{gs_name}] failed to become ready: {err:?}");
            });

        match ready {
            Ok(gs) => Ok(gs),
            Err(err) => {
                if let Err(cleanup_err) = self.drop_gs(&gs_name).await {
                    warn!(
                        "Failed to cleanup GameServer[{gs_name}] after ready timeout: {cleanup_err:?}"
                    );
                }
                Err(err)
            }
        }
    }
}
