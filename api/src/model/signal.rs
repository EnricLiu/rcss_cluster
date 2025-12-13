use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "ty", bound(deserialize = "'de: 'a"))]
pub enum Payload<'a> {
    #[serde(rename = "r")]
    Raw { d: Cow<'a, str> },
    #[serde(rename = "e")]
    Error { e: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Signal<'a> {
    i: Uuid,
    p: Payload<'a>,
}

impl<'a> Signal<'a> {
    pub fn raw_ref(data: &'a str) -> Self {
        Signal {
            i: Uuid::now_v7(),
            p: Payload::Raw {
                d: Cow::Borrowed(data),
            },
        }
    }

    pub fn raw_own(data: String) -> Signal<'static> {
        Signal {
            i: Uuid::now_v7(),
            p: Payload::Raw {
                d: Cow::Owned(data),
            },
        }
    }

    pub fn raw(data: Cow<'a, str>) -> Self {
        Signal {
            i: Uuid::now_v7(),
            p: Payload::Raw { d: data },
        }
    }

    pub fn into_owned(self) -> Signal<'static> {
        let Self { i, p } = self;
        let p = match p {
            Payload::Raw { d } => Payload::Raw {
                d: Cow::Owned(d.into_owned()),
            },
            Payload::Error { e } => Payload::Error { e },
        };

        Signal { i, p }
    }

    pub fn error(error: &impl std::error::Error) -> Self {
        Signal {
            i: Uuid::now_v7(),
            p: Payload::Error {
                e: error.to_string(),
            },
        }
    }
}
