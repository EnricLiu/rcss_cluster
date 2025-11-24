use arcstr::ArcStr;

#[derive(Clone, Debug)]
pub enum ClientSignal {
    Data(ArcStr),
    Shutdown,
}

impl ClientSignal {
    pub fn is_shutdown(&self) -> bool {
        match self {
            ClientSignal::Shutdown => true,
            _ => false,
        }
    }
}
