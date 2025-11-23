use arcstr::ArcStr;

#[derive(Clone, Debug)]
pub enum ClientSignal {
    Data(ArcStr),
    Shutdown,
}