use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum EyeMode {
    On, Off
}
impl EyeMode {
    pub fn encode(self) -> &'static str {
        match self {
            EyeMode::On => "on",
            EyeMode::Off => "off"
        }
    }
    pub fn decode(s: &str) -> Option<Self> {
        match s {
            "on" => Some(EyeMode::On),
            "off" => Some(EyeMode::Off),
            _ => None,
        }
    }
}

impl std::str::FromStr for EyeMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <EyeMode as std::str::FromStr>::Err> {
        Self::decode(s).ok_or(())
    }
}

impl Serialize for EyeMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer,
    {
        serializer.serialize_str(self.encode())
    }
}

impl<'de> Deserialize<'de> for EyeMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        EyeMode::decode(&s).ok_or_else(|| serde::de::Error::custom("invalid EyeMode"))
    }
}
