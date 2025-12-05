use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;
use arcstr::ArcStr;

pub mod trainer;
pub mod player;

pub trait Command {
    type Kind: CommandAny + Send + 'static;
    type Ok: std::fmt::Debug + Send + 'static;
    type Error: std::error::Error + Send + 'static;

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
