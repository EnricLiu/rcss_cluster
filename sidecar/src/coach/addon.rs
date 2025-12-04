use std::fmt::Debug;

use tokio::sync::mpsc;
use arcstr::ArcStr;

use common::client::{RxData, TxData, TxSignal};


pub trait Addon: Debug + Send + 'static {
    fn close(&self) {
        
    }
}

pub trait RawAddon: Addon {
    fn from_raw(
        sig_tx:  mpsc::Sender<TxSignal>,
        data_tx: mpsc::Sender<TxData>,
        data_rx: mpsc::Receiver<RxData>,
    ) -> Self where Self: Sized;
}

pub trait CallerAddon: Addon {
    type Handle: Sync + Send + 'static;
    fn handle(&self) -> Self::Handle;
    
    fn from_caller(
        sig_tx:  mpsc::Sender<TxSignal>,
        caller:  super::resolver::Sender,
    ) -> Self where Self: Sized;
}
