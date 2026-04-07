use std::fmt;
use std::ops::Range;
use std::str::FromStr;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde::de::Visitor;
use common::errors::{BuilderError, BuilderResult};

#[derive(Serialize, Debug, Clone)]
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

/// Characters allowed in provider and model segments of an image string.
///
/// Only ASCII alphanumeric and `-` are permitted.  In particular `_` and `.`
/// are forbidden because the label-serialization layer uses them as
/// [`FIELD_SEP`] and [`IMAGE_SEP`] respectively; allowing them here would
/// make the encoded label ambiguous / non-roundtrippable.
fn is_valid_image_char(c: char) -> bool {
    matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-')
}

impl TryFrom<String> for Image {
    type Error = BuilderError;

    fn try_from(raw: String) -> BuilderResult<Image> {
        let mut parts = raw.split('/');
        if let (Some(provider), Some(model)) = (parts.next(), parts.next()) {
            if parts.next().is_none()
                && !provider.is_empty()
                && !model.is_empty()
                && provider.chars().all(is_valid_image_char)
                && model.chars().all(is_valid_image_char)
            {
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
            expected: "format: {provider}/{model} where each segment matches [A-Za-z0-9-]+".to_string(),
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

impl<'de> Deserialize<'de> for Image {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ImageVisitor;

        impl<'de> Visitor<'de> for ImageVisitor {
            type Value = Image;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string in the format 'provider/model'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Image, E>
            where
                E: de::Error,
            {
                Image::try_from(value).map_err(de::Error::custom)
            }

            fn visit_string<E>(self, value: String) -> Result<Image, E>
            where
                E: de::Error,
            {
                Image::try_from(value).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(ImageVisitor)
    }
}
