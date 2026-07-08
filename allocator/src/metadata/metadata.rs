use serde::{Deserialize, Serialize};
#[cfg(feature = "agones")]
use agones::ObjectMeta;

use common::errors::BuilderError;

use super::labels::Labels;
use super::annotations::Annotations;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MetaData {
    pub labels: Labels,
    pub annotations: Annotations,
}

#[cfg(feature = "agones")]
impl TryFrom<ObjectMeta> for MetaData {
    type Error = BuilderError;

    fn try_from(meta: ObjectMeta) -> Result<Self, Self::Error> {
        let labels = meta.labels.try_into()?;
        let annotations = meta.annotations.try_into()?;

        let ret = Self {
            labels,
            annotations,
        };

        Ok(ret)
    }
}
