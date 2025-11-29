use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum BallPosition {
    InField,
    GoalL,
    GoalR,
    OutOfField,
}

impl BallPosition {
    pub fn encode(self) -> &'static str {
        match self {
            BallPosition::InField => "in_field",
            BallPosition::GoalL => "goal_l",
            BallPosition::GoalR => "goal_r",
            BallPosition::OutOfField => "out_of_field",
        }
    }
    pub fn decode(s: &str) -> Option<Self> {
        match s {
            "in_field" => Some(BallPosition::InField),
            "goal_l" => Some(BallPosition::GoalL),
            "goal_r" => Some(BallPosition::GoalR),
            "out_of_field" => Some(BallPosition::OutOfField),
            _ => None,
        }
    }
}

impl FromStr for BallPosition {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <BallPosition as FromStr>::Err> {
        Self::decode(s).ok_or(())
    }
}
