use std::fmt::Display;

#[derive(Copy, Eq, PartialEq, Clone, Hash, Debug)]
pub enum TeamSide { Left, Right }
impl Display for TeamSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamSide::Left => write!(f, "LEFT"),
            TeamSide::Right => write!(f, "RIGHT"),
        }
    }
}

impl Default for TeamSide {
    fn default() -> Self {
        TeamSide::Left
    }
}