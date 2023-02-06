use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
