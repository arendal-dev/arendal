use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Boolean,
    True,
    False,
    Integer,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
