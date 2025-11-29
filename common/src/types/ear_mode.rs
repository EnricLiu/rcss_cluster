#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum EarMode {
    On, Off
}
impl EarMode {
    pub fn encode(self) -> &'static str {
        match self {
            EarMode::On => "on",
            EarMode::Off => "off"
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
