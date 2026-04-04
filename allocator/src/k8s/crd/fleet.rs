use std::borrow::Cow;
use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::{
    Container, EnvVar, PodSpec, PodTemplateSpec, ResourceRequirements,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use common::errors::{BuilderError, BuilderResult};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fleet {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: ObjectMeta,
    pub spec: FleetSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replicas: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduling: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<FleetStrategy>,
    #[serde(rename = "allocationOverflow", skip_serializing_if = "Option::is_none")]
    pub allocation_overflow: Option<AllocationOverflow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priorities: Option<Vec<Priority>>,
    pub template: GameServerTemplateSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetStrategy {
    #[serde(rename = "type")]
    pub strategy_type: String,
    #[serde(rename = "rollingUpdate", skip_serializing_if = "Option::is_none")]
    pub rolling_update: Option<RollingUpdateStrategy>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollingUpdateStrategy {
    #[serde(rename = "maxSurge", skip_serializing_if = "Option::is_none")]
    pub max_surge: Option<String>,
    #[serde(rename = "maxUnavailable", skip_serializing_if = "Option::is_none")]
    pub max_unavailable: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AllocationOverflow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Priority {
    #[serde(rename = "type")]
    pub priority_type: String,
    pub key: String,
    pub order: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerTemplateSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ObjectMeta>,
    pub spec: GameServerSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<GameServerPortSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthSpec>,
    #[serde(rename = "sdkServer", skip_serializing_if = "Option::is_none")]
    pub sdk_server: Option<SdkServerSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counters: Option<BTreeMap<String, CounterStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<BTreeMap<String, ListStatus>>,
    pub template: PodTemplateSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerPortSpec {
    pub name: String,
    #[serde(rename = "portPolicy")]
    pub port_policy: String,
    #[serde(rename = "containerPort")]
    pub container_port: i32,
    pub protocol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(rename = "initialDelaySeconds", skip_serializing_if = "Option::is_none")]
    pub initial_delay_seconds: Option<i32>,
    #[serde(rename = "periodSeconds", skip_serializing_if = "Option::is_none")]
    pub period_seconds: Option<i32>,
    #[serde(rename = "failureThreshold", skip_serializing_if = "Option::is_none")]
    pub failure_threshold: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SdkServerSpec {
    #[serde(rename = "logLevel", skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(rename = "grpcPort", skip_serializing_if = "Option::is_none")]
    pub grpc_port: Option<i32>,
    #[serde(rename = "httpPort", skip_serializing_if = "Option::is_none")]
    pub http_port: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CounterStatus {
    pub count: i64,
    pub capacity: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListStatus {
    #[serde(default)]
    pub values: Vec<String>,
}

impl kube::Resource for Fleet {
    type DynamicType = ();
    type Scope = kube::core::NamespaceResourceScope;

    fn kind(_: &()) -> Cow<'_, str> {
        "Fleet".into()
    }

    fn group(_: &()) -> Cow<'_, str> {
        "agones.dev".into()
    }

    fn version(_: &()) -> Cow<'_, str> {
        "v1".into()
    }

    fn plural(_: &()) -> Cow<'_, str> {
        "fleets".into()
    }

    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}

const DEFAULT_AGONES_RCSS_CONTAINER: &str = "agones-rcss-server";
const DEFAULT_AGONES_RCSS_CONTAINER_IMAGE: &str = "registry.cn-beijing.aliyuncs.com/nn4h0u/agones-rcss-server:latest";

#[derive(Clone, Debug, Default)]
pub(crate) struct ContainerBuilder {
    name: Option<String>,
    image: Option<String>,
    requests: BTreeMap<String, Quantity>,
    limits: BTreeMap<String, Quantity>,
    env: Vec<EnvVar>,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default_rcss() -> Self {
        let mut ret = Self::new();
        
        ret.with_name(DEFAULT_AGONES_RCSS_CONTAINER)
            .with_image(DEFAULT_AGONES_RCSS_CONTAINER_IMAGE)
            .with_request("memory", "128Mi")
            .with_request("cpu", "100m")
            .with_limit("memory", "768Mi")
            .with_limit("cpu", "500m");
        
        ret
    }

    pub fn with_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_image(&mut self, image: impl Into<String>) -> &mut Self {
        self.image = Some(image.into());
        self
    }

    pub fn with_request(&mut self, key: impl Into<String>, val: impl Into<String>) -> &mut Self {
        self.requests.insert(key.into(), Quantity(val.into()));
        self
    }

    pub fn with_limit(&mut self, key: impl Into<String>, val: impl Into<String>) -> &mut Self {
        self.limits.insert(key.into(), Quantity(val.into()));
        self
    }

    pub fn with_env(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.env.push(EnvVar {
            name: name.into(),
            value: Some(value.into()),
            value_from: None,
        });
        self
    }

    pub fn with_envs(&mut self, iter: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> &mut Self {
        for (k, v) in iter {
            self.with_env(k, v);
        }
        self
    }

    pub fn build_into(self) -> BuilderResult<Container> {
        let name = self.name.ok_or(BuilderError::MissingField { field: "container.name" })?;
        let image = self.image.ok_or(BuilderError::MissingField { field: "container.image" })?;

        let resources = if self.requests.is_empty() && self.limits.is_empty() {
            None
        } else {
            Some(ResourceRequirements {
                requests: if self.requests.is_empty() { None } else { Some(self.requests) },
                limits: if self.limits.is_empty() { None } else { Some(self.limits) },
                ..Default::default()
            })
        };

        let env = if self.env.is_empty() { None } else { Some(self.env) };

        Ok(Container {
            name,
            image: Some(image),
            resources,
            env,
            ..Default::default()
        })
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FleetBuilder {
    name: Option<String>,
    replicas: Option<i32>,
    scheduling: Option<String>,
    labels: Option<HashMap<String, String>>,
    annotations: Option<HashMap<String, String>>,
    containers: Vec<Container>,
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

    pub fn with_scheduling(&mut self, scheduling: impl Into<String>) -> &mut Self {
        self.scheduling = Some(scheduling.into());
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

    pub fn add_container(&mut self, cb: ContainerBuilder) -> BuilderResult<&mut Self> {
        self.containers.push(cb.build_into()?);
        Ok(self)
    }

    pub fn build_into(self) -> BuilderResult<Fleet> {
        let name = self.name.ok_or(BuilderError::MissingField { field: "fleet.name" })?;

        if self.containers.is_empty() {
            return Err(BuilderError::MissingField { field: "fleet.containers" });
        }

        let replicas = self.replicas.unwrap_or(1);
        let scheduling = self.scheduling.unwrap_or_else(|| "Packed".to_string());

        let gs_template_metadata = match (&self.labels, &self.annotations) {
            (None, None) => None,
            _ => Some(ObjectMeta {
                labels: self.labels.map(|m| m.into_iter().collect()),
                annotations: self.annotations.map(|m| m.into_iter().collect()),
                ..Default::default()
            }),
        };

        Ok(Fleet {
            api_version: "agones.dev/v1".to_string(),
            kind: "Fleet".to_string(),
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            spec: FleetSpec {
                replicas: Some(replicas),
                scheduling: Some(scheduling),
                strategy: Some(FleetStrategy {
                    strategy_type: "RollingUpdate".to_string(),
                    rolling_update: Some(RollingUpdateStrategy {
                        max_surge: Some("25%".to_string()),
                        max_unavailable: Some("25%".to_string()),
                    }),
                }),
                allocation_overflow: None,
                priorities: Some(vec![
                    Priority {
                        priority_type: "Counter".to_string(),
                        key: "rooms".to_string(),
                        order: "Descending".to_string(),
                    },
                    Priority {
                        priority_type: "List".to_string(),
                        key: "players".to_string(),
                        order: "Ascending".to_string(),
                    },
                ]),
                template: GameServerTemplateSpec {
                    metadata: gs_template_metadata,
                    spec: GameServerSpec {
                        ports: Some(vec![
                            GameServerPortSpec {
                                name: "default".to_string(),
                                port_policy: "Dynamic".to_string(),
                                container_port: 55555,
                                protocol: "TCP".to_string(),
                            },
                            GameServerPortSpec {
                                name: "player".to_string(),
                                port_policy: "Dynamic".to_string(),
                                container_port: 6000,
                                protocol: "UDP".to_string(),
                            },
                            GameServerPortSpec {
                                name: "trainer".to_string(),
                                port_policy: "Dynamic".to_string(),
                                container_port: 6001,
                                protocol: "UDP".to_string(),
                            },
                            GameServerPortSpec {
                                name: "coach".to_string(),
                                port_policy: "Dynamic".to_string(),
                                container_port: 6002,
                                protocol: "UDP".to_string(),
                            },
                        ]),
                        health: Some(HealthSpec {
                            disabled: Some(false),
                            initial_delay_seconds: Some(30),
                            period_seconds: Some(30),
                            failure_threshold: Some(3),
                        }),
                        sdk_server: Some(SdkServerSpec {
                            log_level: Some("Info".to_string()),
                            grpc_port: Some(9357),
                            http_port: Some(9358),
                        }),
                        counters: Some(BTreeMap::from([(
                            "rooms".to_string(),
                            CounterStatus {
                                count: 0,
                                capacity: 100,
                            },
                        )])),
                        lists: Some(BTreeMap::from([(
                            "players".to_string(),
                            ListStatus { values: vec![] },
                        )])),
                        template: PodTemplateSpec {
                            metadata: None,
                            spec: Some(PodSpec {
                                containers: self.containers,
                                ..Default::default()
                            }),
                        },
                    },
                },
            },
        })
    }
}