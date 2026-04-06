use std::collections::{BTreeMap, HashMap};

use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

use common::errors::{BuilderError, BuilderResult};

use super::crd::Fleet;
use super::template::fleet_template;

#[derive(Clone, Debug, Default)]
pub struct FleetBuilder {
    name: Option<String>,
    replicas: Option<i32>,
    labels: Option<HashMap<String, String>>,
    annotations: Option<HashMap<String, String>>,
}

impl FleetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_replicas(&mut self, replicas: i32) -> &mut Self {
        self.replicas = Some(replicas);
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

    pub fn build_into(self) -> BuilderResult<Fleet> {
        let name = self.name.ok_or(BuilderError::MissingField { field: "fleet.name" })?;

        let template = fleet_template();
        let mut fleet = template.clone();

        fleet.metadata = ObjectMeta {
            name: Some(name),
            ..Default::default()
        };

        if let Some(replicas) = self.replicas {
            fleet.spec.replicas = Some(replicas);
        }

        // Merge labels & annotations into the GameServer template metadata
        if self.labels.is_some() || self.annotations.is_some() {
            let gs_meta = fleet.spec.template.metadata.get_or_insert_with(Default::default);

            if let Some(builder_labels) = self.labels {
                let labels = gs_meta.labels.get_or_insert_with(BTreeMap::new);
                for (k, v) in builder_labels {
                    labels.insert(k, v);
                }
            }

            if let Some(builder_annotations) = self.annotations {
                let annotations = gs_meta.annotations.get_or_insert_with(BTreeMap::new);
                for (k, v) in builder_annotations {
                    annotations.insert(k, v);
                }
            }
        }

        Ok(fleet)
    }
}