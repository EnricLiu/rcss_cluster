use std::ops::Range;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use common::errors::{BuilderError, BuilderResult};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct Image {
    #[serde(rename = "image")]
    pub raw: String,

    #[serde(skip)]
    provider: Range<usize>,
    #[serde(skip)]
    model: Range<usize>,
}

impl Image {
    pub fn provider(&self) -> &str {
        &self.raw[self.provider.clone()]
    }

    pub fn model(&self) -> &str {
        &self.raw[self.model.clone()]
    }
}

impl TryFrom<String> for Image {
    type Error = BuilderError;

    fn try_from(raw: String) -> BuilderResult<Image> {
        let mut parts = raw.split('/');
        if let (Some(provider), Some(_)) = (parts.next(), parts.next()) {
            if parts.next().is_none() {
                let split_pos = provider.len();
                let len = raw.len();
                return Ok(Image {
                    raw,
                    provider: 0..split_pos,
                    model: (split_pos+1)..len,
                });
            }
        }

        Err(BuilderError::InvalidValue {
            field: "image",
            value: raw.to_string(),
            expected: r"format: /^\w+/(\w+|\*):?\w*?$/".to_string(),
        })
    }
}

impl TryFrom<&str> for Image {
    type Error = BuilderError;

    fn try_from(raw: &str) -> BuilderResult<Image> {
        raw.to_string().try_into()
    }
}

impl FromStr for Image {
    type Err = BuilderError;

    fn from_str(s: &str) -> BuilderResult<Self> {
        s.try_into()
    }
}
