use std::collections::HashMap;
use std::fmt::Debug;
use std::net::IpAddr;

use common::errors::{BuilderError, BuilderResult};
use super::crd::GameServerPort;

#[derive(Debug, Clone)]
pub struct GsAllocation {
    pub name: String,
    pub pod: IpAddr,
    pub host: IpAddr,
    pub ports: HashMap<String, u16>,
}

impl GsAllocation {
    pub fn builder() -> GsAllocationBuilder {
        GsAllocationBuilder::new()
    }
}

#[derive(Default, Debug, Clone)]
pub struct GsAllocationBuilder {
    name: Option<String>,
    pod: Option<IpAddr>,
    host: Option<IpAddr>,
    ports: HashMap<String, u16>,
}
impl GsAllocationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_host(&mut self, host: Option<&String>) -> BuilderResult<&mut Self> {
        let host = host.ok_or(BuilderError::MissingField { field: "host" })?;
        let host = host.parse().map_err(|_|BuilderError::InvalidValue {
            field: "host",
            value: host.to_string(),
            expected: "an IpAddr".to_string(),
        })?;

        self.host = Some(host);
        Ok(self)
    }

    pub fn with_host(&mut self, host: impl Into<IpAddr>) -> &mut Self {
        self.host = Some(host.into());
        self
    }
    
    pub fn set_pod_ip(&mut self, pod_ip: Option<IpAddr>) -> &mut Self {
        self.pod = pod_ip;
        self
    }

    pub fn parse_ports(&mut self, ports: Vec<GameServerPort>) -> &mut Self {
        for port in ports {
            self.add_port(port.name, port.port);
        }
        self
    }

    pub fn add_port(&mut self, name: impl Into<String>, port: u16) -> &mut Self {
        self.ports.insert(name.into(), port);
        self
    }

    pub fn with_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn set_name(&mut self, name: Option<String>) -> &mut Self {
        self.name = name;
        self
    }

    pub fn build_into(self) -> BuilderResult<GsAllocation> {
        let name = self.name.ok_or(BuilderError::MissingField { field: "name" })?;
        let pod = self.pod.ok_or(BuilderError::MissingField { field: "pod" })?;
        let host = self.host.ok_or(BuilderError::MissingField { field: "host" })?;
        let ports = self.ports;
        if ports.is_empty() {
            return Err(BuilderError::MissingField { field: "ports" });
        }

        Ok(GsAllocation { name, pod, host, ports })
    }
}
