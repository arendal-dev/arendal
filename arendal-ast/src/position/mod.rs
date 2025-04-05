use std::fmt;

use crate::input::StrRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    NoPosition,
    String(StrRange),
}

impl Position {
    pub fn merge(&self, other: &Position) -> Position {
        match self {
            Self::NoPosition => Self::NoPosition,
            Self::String(r1) => match other {
                Self::NoPosition => Self::NoPosition,
                Self::String(r2) => match r1.merge(r2) {
                    Ok(r) => Self::String(r),
                    _ => Self::NoPosition,
                },
            },
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Position::NoPosition => Ok(()),
            Position::String(range) => write!(f, "@{}", range),
        }
    }
}
