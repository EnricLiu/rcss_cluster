use crate::schema::Schema;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "agent", rename_all = "lowercase")]
pub enum AgentV1 {
    SSP {
        image: String,
        grpc_host: IpAddr,
        grpc_port: u16,
    },
}

impl AgentV1 {
    pub fn image(&self) -> &str {
        match self {
            AgentV1::SSP { image, .. } => image,
        }
    }

    pub fn grpc_host(&self) -> IpAddr {
        match self {
            AgentV1::SSP { grpc_host, .. } => *grpc_host,
        }
    }

    pub fn grpc_port(&self) -> u16 {
        match self {
            AgentV1::SSP { grpc_port, .. } => *grpc_port,
        }
    }
}

impl Schema for AgentV1 {
    fn verify(&self) -> Result<(), &'static str> {
        let policy = match self {
            AgentV1::SSP { image, .. } => image,
        };

        let mut res = policy.split('/');
        if let Some(provider) = res.next() {
            if let Some(_policy_name) = res.next() {
                return Ok(());
            }

            if provider == "*" {
                return Ok(());
            }

            return Err(r"Invalid policy name, should be in pattern /^\w+/(\w+|\*):?\w*?$/");
        }

        Ok(())
    }
}
