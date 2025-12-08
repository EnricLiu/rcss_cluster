use std::any::Any;
use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;
use arcstr::ArcStr;
use serde::Serialize;

pub mod trainer;
pub mod player;

pub trait Command {
    type Kind: CommandAny + Send + 'static;
    type Ok: CommandOk;
    type Error: CommandError;

    fn kind(&self) -> Self::Kind;
    fn encode(&self) -> ArcStr;
    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> where Self: Sized {
        None // default never ok
    }
    fn parse_ret_err(tokens: &[&str]) -> Option<Self::Error> where Self: Sized {
        None // default never error
    }
}

pub trait CommandAny: Hash + Eq + Clone + Debug + Send + Sync + 'static {
    fn encode(&self) -> ArcStr;
    fn decode(s: &str) -> Option<Self> where Self: Sized;
    fn parse_ret_ok(&self, tokens: &[&str]) -> Option<Box<dyn Any + Send>>;
    fn parse_ret_err(&self, tokens: &[&str]) -> Option<Box<dyn Any + Send>>;
}

pub trait CommandOk: Serialize + Debug + Send + 'static {}
impl <T> CommandOk for T where T: Serialize + Debug + Send + 'static {}
pub trait CommandError: Error + Send + 'static {}
impl <E> CommandError for E where E: Error + Send + 'static {}

pub type CommandResultOk<C> = <C as Command>::Ok;
pub type CommandResultError<C> = <C as Command>::Error;
pub type CommandResult<C> = Result<<C as Command>::Ok, <C as Command>::Error>;
