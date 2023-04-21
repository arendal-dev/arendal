use std::fmt;

use im::HashMap;

use crate::error::{Error, Errors, Loc, Result};
use crate::symbol::{FQSym, Path, Pkg, Symbol};
use crate::types::{Singleton, Type};
use crate::visibility::{self, Visibility, Visible};
use crate::Integer;

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    None,
    True,
    False,
    Integer(Integer),
    Singleton(Singleton),
}

impl Value {
    pub fn integer(value: Integer) -> Self {
        Self::Integer(value)
    }

    pub fn int64(value: i64) -> Self {
        Self::integer(value.into())
    }

    pub fn boolean(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }

    pub fn clone_type(&self) -> Type {
        match self {
            Self::None => Type::None,
            Self::True => Type::True,
            Self::False => Type::False,
            Self::Integer(_) => Type::Integer,
            Self::Singleton(t) => Type::Singleton(t.clone()),
        }
    }

    pub fn as_integer(self) -> Option<Integer> {
        match self {
            Self::Integer(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_boolean(self) -> Option<bool> {
        match self {
            Self::True => Some(true),
            Self::False => Some(false),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("None"),
            Self::True => f.write_str("True"),
            Self::False => f.write_str("False"),
            Self::Integer(value) => value.fmt(f),
            Self::Singleton(t) => t.fmt(f),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Values {
    values: HashMap<FQSym, Visible<Value>>,
}

impl Values {
    pub(crate) fn get(&self, symbol: &FQSym) -> Option<Visible<Value>> {
        self.values.get(symbol).cloned()
    }

    pub(crate) fn set(
        &mut self,
        loc: Loc,
        symbol: FQSym,
        visibility: Visibility,
        value: Value,
    ) -> Result<()> {
        if self.values.contains_key(&symbol) {
            Errors::err(loc, ValueError::DuplicateValue(symbol))
        } else {
            self.values.insert(symbol, Visible::new(visibility, value));
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum ValueError {
    DuplicateValue(FQSym),
}

impl Error for ValueError {}
