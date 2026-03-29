use std::fmt::Display;
use std::ops::Deref;
use common::errors::{BuilderError, BuilderResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct Unum(u8);

impl Display for Unum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Unum {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Unum {
    pub fn new(unum: u8) -> BuilderResult<Self> {
        if unum  > 11 {
            return Err(BuilderError::InvalidValue {
                field: "unum",
                value: unum.to_string(),
                expected: "[0, 11]".to_string(),
            })
        }
        
        Ok(Self(unum))
    }
}

pub fn unum(unum: u8) -> BuilderResult<Unum> {
    Unum::new(unum)
}
