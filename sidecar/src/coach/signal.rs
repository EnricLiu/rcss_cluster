use std::borrow::Cow;
use arcstr::{ArcStr, format, literal};
use common::types;

pub enum CoachSignal {
    ChangeMode {
        play_mode: types::PlayMode,
    },
    Move {
        todo: (),
    },
    CheckBall,
    Start,
    Recover,
    Ear {
        mode: types::EarMode,
    },
    Init {
        version: usize,
    },
    Look,
    Eye {
        mode: types::EyeMode,
    },
    TeamNames,
}

impl CoachSignal {
    pub fn encode(&self) -> ArcStr {
        use CoachSignal::*;
        match self {
            ChangeMode { play_mode } => {
                format!("(change_mode {})", play_mode.encode())
            },
            Move { .. } => {
                todo!()
            }
            CheckBall => literal!("(check_ball)"),
            Start => literal!("(start)"),
            Recover => literal!("(recover)"),
            Ear { mode } => {
                format!("(ear {})", mode.encode())
            },
            Init { version } => {
                format!("(init version {})", version)
            },
            Look => literal!("(look)"),
            Eye { mode } => {
                format!("(eye {})", mode.encode())
            },
            TeamNames => literal!("(team_names)"),
        }
    }
}
