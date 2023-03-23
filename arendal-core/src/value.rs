use std::fmt;

use crate::types::Type;
use crate::Integer;

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Int(Integer),
    True,
    False,
}

use Value::*;

impl Value {
    pub fn integer(value: Integer) -> Self {
        Int(value)
    }

    pub fn int64(value: i64) -> Self {
        Self::integer(value.into())
    }

    pub fn boolean(value: bool) -> Self {
        if value {
            True
        } else {
            False
        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            Int(_) => Type::integer(),
            True => Type::boolean_true(),
            False => Type::boolean_false(),
        }
    }

    pub fn as_integer(self) -> Option<Integer> {
        match self {
            Int(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_boolean(self) -> Option<bool> {
        match self {
            True => Some(true),
            False => Some(false),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Int(value) => value.fmt(f),
            True => f.write_str("True"),
            False => f.write_str("False"),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
