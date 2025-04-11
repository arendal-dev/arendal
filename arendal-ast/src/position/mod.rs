use std::fmt::{self, Debug};

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

// Used mostly for testing, for types that skip the position when testing equality
pub trait EqNoPosition: Debug {
    fn eq_nopos(&self, other: &Self) -> bool;

    fn assert_eq_nopos(&self, expected: &Self) {
        if !self.eq_nopos(expected) {
            panic!(
                "Equality (no position) assertion failed!\nActual: {:?}\nExpected: {:?}",
                self, expected
            )
        }
    }
}

impl<T: EqNoPosition> EqNoPosition for Vec<T> {
    fn eq_nopos(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (s, o) in self.iter().zip(other.iter()) {
            if !s.eq_nopos(o) {
                return false;
            }
        }
        true
    }
}
