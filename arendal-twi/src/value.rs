use std::fmt;

use core::types::Type;
use core::Integer;

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Int(Integer),
}

use Value::*;

impl Value {
    pub fn integer(value: Integer) -> Self {
        Int(value)
    }

    pub fn int64(value: i64) -> Self {
        Self::integer(value.into())
    }

    pub fn get_type(&self) -> Type {
        match self {
            Int(_) => Type::integer(),
        }
    }

    pub fn as_integer(self) -> Option<Integer> {
        match self {
            Int(v) => Some(v),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Int(value) => value.fmt(f),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Int(value) => value.fmt(f),
        }
    }
}
