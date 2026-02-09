use crate::schema::Schema;

#[derive(Clone, Debug)]
pub enum Policy {
    Bot {
        provider: String
    },
    Agent,
}

impl Policy {
    pub fn bot(policy: String) -> Policy {
        Policy::Bot { 
            provider: policy,
        }
    }

    pub fn agent() -> Policy {
        Policy::Agent
    }
}

impl Schema for Policy {
    fn verify(&self) -> Result<(), &'static str> {
        let policy = match self {
            Policy::Bot { provider } => provider,
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
