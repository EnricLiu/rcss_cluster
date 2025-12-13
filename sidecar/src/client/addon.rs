use std::fmt::Debug;

use arcstr::ArcStr;
use tokio::sync::mpsc;

use common::client::{RxData, TxData, TxSignal};
use common::command::CommandAny;

pub trait Addon: Debug + Send + Sync + 'static {
    fn close(&self) {}
}

pub trait RawAddon: Addon {
    fn from_raw(
        sig_tx: mpsc::Sender<TxSignal>,
        data_tx: mpsc::Sender<TxData>,
        data_rx: mpsc::Receiver<RxData>,
    ) -> Self
    where
        Self: Sized;
}

pub trait CallerAddon<CMD: CommandAny>: Addon {
    type Handle: Sync + Send + 'static;
    fn handle(&self) -> Self::Handle;

    fn from_caller(
        sig_tx: mpsc::Sender<TxSignal>,
        caller: super::resolver::Sender<CMD, TxData, RxData>,
    ) -> Self
    where
        Self: Sized;
}
