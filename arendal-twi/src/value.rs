use ast::Type;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Integer(num::Integer),
}

use Value::*;

impl Value {
    pub fn integer(value: num::Integer) -> Self {
        Integer(value)
    }

    pub fn int64(value: i64) -> Self {
        Self::integer(value.into())
    }

    pub fn get_type(&self) -> Type {
        match self {
            Integer(_) => Type::Integer,
        }
    }

    pub fn as_integer(self) -> Option<num::Integer> {
        match self {
            Integer(v) => Some(v),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Integer(value) => value.fmt(f),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Integer(value) => value.fmt(f),
        }
    }
}
