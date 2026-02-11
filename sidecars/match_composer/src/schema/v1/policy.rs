use serde::{Deserialize, Serialize};
use crate::schema::Schema;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Policy {
    Bot {
        image: String
    },
    Agent,
}

impl Policy {
    pub fn bot(image: String) -> Policy {
        Policy::Bot {
            image,
        }
    }

    pub fn agent() -> Policy {
        Policy::Agent
    }
}

impl Schema for Policy {
    fn verify(&self) -> Result<(), &'static str> {
        let policy = match self {
            Policy::Bot { image } => image,
            Policy::Agent => return Ok(()),
        };
        
        let mut res = policy.split('/');
        if let Some(provider) = res.next() {
            if let Some(_policy_name) = res.next() {
                return Ok(())
            }

            if provider == "*" {
                return Ok(())
            }

            return Err(r"Invalid policy name, should be in pattern /^\w+/(\w+|\*):?w*?$/")
        }
        
        Ok(())
    }
}
