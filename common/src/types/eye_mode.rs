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
