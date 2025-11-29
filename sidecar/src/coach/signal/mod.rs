mod change_mode;
mod check_ball;
mod ear;
mod eye;
mod init;
mod look;
mod r#move;
mod recover;
mod start;
mod team_names;

use arcstr::{ArcStr, format, literal};

pub use change_mode::SignalChangeMode as ChangeMode;
pub use check_ball::SignalCheckBall as CheckBall;
pub use r#move::SignalMove as Move;
pub use ear::SignalEar as Ear;
pub use eye::SignalEye as Eye;
pub use init::SignalInit as Init;
pub use look::SignalLook as Look;
pub use recover::SignalRecover as Recover;
pub use start::SignalStart as Start;
pub use team_names::SignalTeamNames as TeamNames;


pub trait Signal {
    type Ok: std::fmt::Debug + Send + 'static;
    type Error: std::error::Error + Send + 'static;
    
    fn kind(&self) -> SignalKind;
    fn encode(&self) -> ArcStr;
    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> where Self: Sized {
        None // default never ok
    }
    fn parse_ret_err(tokens: &[&str]) -> Option<Self::Error> where Self: Sized {
        None // default never error
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum SignalKind {
    ChangeMode,
    Move,
    CheckBall,
    Start,
    Recover,
    Ear,
    Init,
    Look,
    Eye,
    TeamNames,
}

