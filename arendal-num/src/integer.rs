use std::fmt;
use std::num::ParseIntError;
use std::ops;
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Integer {
    value: i64, // Temporary
}

impl FromStr for Integer {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: i64 = s.parse()?;
        Ok(Integer { value })
    }
}

impl From<i64> for Integer {
    fn from(value: i64) -> Self {
        Integer { value }
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl fmt::Debug for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl ops::Add for Integer {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        (self.value + other.value).into()
    }
}
