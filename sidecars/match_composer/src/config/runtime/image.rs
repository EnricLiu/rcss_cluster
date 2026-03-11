use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct ImageMeta {
    pub provider: String,
    pub model: String,
    pub path: Box<std::path::Path>,
}

#[derive(Clone, Debug)]
pub struct ImageQuery {
    pub provider: String,
    pub model: String,
}

impl TryFrom<&str> for ImageQuery {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split('/');
        if let (Some(provider), Some(model)) = (parts.next(), parts.next()) {
            if parts.next().is_some() {
                return Err(());
            }

            return Ok(ImageQuery {
                provider: provider.to_string(),
                model: model.to_string(),
            });
        }

        Err(())
    }
}
