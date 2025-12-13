pub mod change_mode;
pub mod check_ball;
pub mod ear;
pub mod eye;
pub mod init;
pub mod look;
pub mod r#move;
pub mod recover;
pub mod start;
pub mod team_names;

pub use change_mode::CommandChangeMode as ChangeMode;
pub use check_ball::CommandCheckBall as CheckBall;
pub use ear::CommandEar as Ear;
pub use eye::CommandEye as Eye;
pub use init::CommandInit as Init;
pub use look::CommandLook as Look;
pub use r#move::CommandMove as Move;
pub use recover::CommandRecover as Recover;
pub use start::CommandStart as Start;
pub use team_names::CommandTeamNames as TeamNames;

use arcstr::{ArcStr, literal};
use std::any::Any;

use super::{Command, CommandAny};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum TrainerCommand {
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

impl CommandAny for TrainerCommand {
    fn encode(&self) -> ArcStr {
        match self {
            TrainerCommand::ChangeMode => literal!("change_mode"),
            TrainerCommand::Move => literal!("move"),
            TrainerCommand::CheckBall => literal!("check_ball"),
            TrainerCommand::Start => literal!("start"),
            TrainerCommand::Recover => literal!("recover"),
            TrainerCommand::Ear => literal!("ear"),
            TrainerCommand::Init => literal!("init"),
            TrainerCommand::Look => literal!("look"),
            TrainerCommand::Eye => literal!("eye"),
            TrainerCommand::TeamNames => literal!("team_names"),
        }
    }
    fn decode(s: &str) -> Option<Self> {
        match s {
            "change_mode" => Some(TrainerCommand::ChangeMode),
            "move" => Some(TrainerCommand::Move),
            "check_ball" => Some(TrainerCommand::CheckBall),
            "start" => Some(TrainerCommand::Start),
            "recover" => Some(TrainerCommand::Recover),
            "ear" => Some(TrainerCommand::Ear),
            "init" => Some(TrainerCommand::Init),
            "look" => Some(TrainerCommand::Look),
            "eye" => Some(TrainerCommand::Eye),
            "team_names" => Some(TrainerCommand::TeamNames),
            _ => None,
        }
    }
    fn parse_ret_ok(&self, tokens: &[&str]) -> Option<Box<dyn Any + Send>> {
        match self {
            TrainerCommand::ChangeMode => {
                ChangeMode::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
            TrainerCommand::Move => {
                Move::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
            TrainerCommand::CheckBall => {
                CheckBall::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
            TrainerCommand::Start => {
                Start::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
            TrainerCommand::Recover => {
                Recover::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
            TrainerCommand::Ear => {
                Ear::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
            TrainerCommand::Init => {
                Init::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
            TrainerCommand::Look => {
                Look::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
            TrainerCommand::Eye => {
                Eye::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
            TrainerCommand::TeamNames => {
                TeamNames::parse_ret_ok(tokens).map(|r| Box::new(r) as Box<dyn Any + Send>)
            }
        }
    }
    fn parse_ret_err(&self, tokens: &[&str]) -> Option<Box<dyn Any + Send>> {
        match self {
            TrainerCommand::ChangeMode => {
                ChangeMode::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
            TrainerCommand::Move => {
                Move::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
            TrainerCommand::CheckBall => {
                CheckBall::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
            TrainerCommand::Start => {
                Start::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
            TrainerCommand::Recover => {
                Recover::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
            TrainerCommand::Ear => {
                Ear::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
            TrainerCommand::Init => {
                Init::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
            TrainerCommand::Look => {
                Look::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
            TrainerCommand::Eye => {
                Eye::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
            TrainerCommand::TeamNames => {
                TeamNames::parse_ret_err(tokens).map(|e| Box::new(e) as Box<dyn Any + Send>)
            }
        }
    }
}

// impl FromStr for TrainerCommand {
//     type Err = ();
//     fn from_str(s: &str) -> Result<Self, <TrainerCommand as FromStr>::Err> {
//         TrainerCommand::decode(s).ok_or(())
//     }
// }
