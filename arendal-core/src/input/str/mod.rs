use crate::problem;
use crate::ArcStr;

#[derive(Clone, Debug)]
pub enum Origin {
    String,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub input: ArcStr,
    pub origin: Origin,
}

impl Input {
    pub fn from_string(input: ArcStr) -> Input {
        Input {
            input,
            origin: Origin::String,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Position {
    bytes_from_start: usize,
    line: usize,
    line_start_bytes: usize,
    character: usize,
    character_bytes: usize,
}

impl problem::Position for Position {}

pub type Errors = problem::Errors<Position>;
pub type Warnings = problem::Warnings<Position>;
pub type Output<T> = problem::Output<Position, T>;
pub type Result<T> = problem::Result<Position, T>;
