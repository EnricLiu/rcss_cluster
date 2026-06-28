use std::collections::{BTreeMap, HashMap};

use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

use common::errors::{BuilderError, BuilderResult};

use super::crd::GameServer;
use super::template::gs_template;

#[derive(Clone, Debug, Default)]
pub struct GameServerBuilder {
    name: Option<String>,
    labels: Option<HashMap<String, String>>,
    annotations: Option<HashMap<String, String>>,
}

impl GameServerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_labels(&mut self, labels: HashMap<String, String>) -> &mut Self {
        self.labels = Some(labels);
        self
    }

    pub fn with_annotations(&mut self, annotations: HashMap<String, String>) -> &mut Self {
        self.annotations = Some(annotations);
        self
    }

    pub fn build_into(self) -> BuilderResult<GameServer> {
        let name = self.name.ok_or(BuilderError::MissingField { field: "gs.name" })?;

        let template = gs_template();
        let mut gs = template.clone();

        gs.metadata = ObjectMeta {
            name: Some(name),
            ..Default::default()
        };

        if self.labels.is_some() || self.annotations.is_some() {
            if let Some(builder_labels) = self.labels {
                let labels = gs.metadata.labels.get_or_insert_with(BTreeMap::new);
                for (k, v) in builder_labels {
                    labels.insert(k, v);
                }
            }

            if let Some(builder_annotations) = self.annotations {
                let annotations = gs.metadata.annotations.get_or_insert_with(BTreeMap::new);
                for (k, v) in builder_annotations {
                    annotations.insert(k, v);
                }
            }
        }

        Ok(gs)
    }
}
