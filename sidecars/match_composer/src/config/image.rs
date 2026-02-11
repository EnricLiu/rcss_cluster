use crate::schema::v1::Policy;

#[derive(Clone, Debug)]
pub struct ImageConfig {
    pub provider: String,
    pub model: String,
}

impl TryFrom<Policy> for ImageConfig {
    type Error = ();

    fn try_from(value: Policy) -> Result<Self, Self::Error> {
        let policy = match value {
            Policy::Bot { image } => image,
            _ => return Err(()),
        };

        let mut parts = policy.split('/');
        if  let Some(provider) = parts.next() &&
            let Some(model) = parts.next() {

            if parts.next().is_some() {
                return Err(())
            }

            return Ok(ImageConfig {
                provider: provider.to_string(),
                model: model.to_string(),
            })

        }

        Err(())
    }
}