use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum EarMode {
    On,
    Off,
}
impl EarMode {
    pub fn encode(self) -> &'static str {
        match self {
            EarMode::On => "on",
            EarMode::Off => "off",
        }
    }
    pub fn decode(s: &str) -> Option<Self> {
        match s {
            "on" => Some(EarMode::On),
            "off" => Some(EarMode::Off),
            _ => None,
        }
    }
}

impl std::str::FromStr for EarMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <EarMode as std::str::FromStr>::Err> {
        Self::decode(s).ok_or(())
    }
}

impl Serialize for EarMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.encode())
    }
}

impl<'de> Deserialize<'de> for EarMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        EarMode::decode(&s).ok_or_else(|| serde::de::Error::custom("invalid EarMode"))
    }
}
