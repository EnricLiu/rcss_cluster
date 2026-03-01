use serde::{Deserialize, Serialize};
use crate::schema::Schema;
use crate::schema::v1::AgentV1;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PolicyV1 {
    Bot {
        image: String
    },
    Agent(AgentV1),
}

impl PolicyV1 {
    pub fn bot(image: String) -> PolicyV1 {
        PolicyV1::Bot {
            image,
        }
    }
    
    pub fn helios_base() -> PolicyV1 {
        PolicyV1::Bot {
            image: "HELIOS/helios-base".to_string(),
        }
    }

    pub fn ssp(image: String, grpc_host: std::net::Ipv4Addr, grpc_port: u16) -> PolicyV1 {
        PolicyV1::Agent(AgentV1::SSP {
            image,
            grpc_host,
            grpc_port,
        })
    }
    
    pub fn image(&self) -> &str {
        match self {
            PolicyV1::Bot { image } => image,
            PolicyV1::Agent(agent) => agent.image(),
        }
    }
}

impl Schema for PolicyV1 {
    fn verify(&self) -> Result<(), &'static str> {
        let policy = match self {
            PolicyV1::Bot { image } => image,
            PolicyV1::Agent(agent) => return agent.verify(),
        };
        
        let mut res = policy.split('/');
        if let Some(provider) = res.next() {
            if let Some(_policy_name) = res.next() {
                return Ok(())
            }

            if provider == "*" {
                return Ok(())
            }

            return Err(r"Invalid policy name, should be in pattern /^\w+/(\w+|\*):?\w*?$/")
        }
        
        Ok(())
    }
}
