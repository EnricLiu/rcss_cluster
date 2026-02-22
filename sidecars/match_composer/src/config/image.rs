use std::path::Path;
use crate::schema::v1::PolicyV1;

#[derive(Clone, Debug)]
pub struct ImageConfig {
    pub provider: String,
    pub model: String,
    pub path: Box<Path>,
}


#[derive(Clone, Debug)]
pub struct ImageQuery {
    pub provider: String,
    pub model: String,
}

impl TryFrom<PolicyV1> for ImageQuery {
    type Error = ();

    fn try_from(value: PolicyV1) -> Result<Self, Self::Error> {
        let policy = match value {
            PolicyV1::Bot { image } => image,
            _ => return Err(()),
        };

        let mut parts = policy.split('/');
        if  let Some(provider) = parts.next() &&
            let Some(model) = parts.next() {

            if parts.next().is_some() {
                return Err(())
            }

            return Ok(ImageQuery {
                provider: provider.to_string(),
                model: model.to_string(),
            })

        }

        Err(())
    }
}