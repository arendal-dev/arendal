use std::fmt;

use im::HashMap;

use crate::error::{Error, Loc, Result};
use crate::symbol::FQSym;
use crate::types::Type;
use crate::visibility::{Visibility, Visible};
use crate::Integer;

#[derive(Clone, PartialEq, Eq)]
pub struct Value {
    pub loc: Loc,
    value: Val,
}

#[derive(Clone, PartialEq, Eq)]
enum Val {
    None,
    True,
    False,
    Integer(Integer),
    Singleton(Type),
}

impl Value {
    fn new(loc: &Loc, value: Val) -> Self {
        Value {
            loc: loc.clone(),
            value,
        }
    }

    pub fn v_none(loc: &Loc) -> Self {
        Self::new(loc, Val::None)
    }

    pub fn v_true(loc: &Loc) -> Self {
        Self::new(loc, Val::True)
    }

    pub fn v_false(loc: &Loc) -> Self {
        Self::new(loc, Val::False)
    }

    pub fn integer(loc: &Loc, value: Integer) -> Self {
        Self::new(loc, Val::Integer(value))
    }

    pub fn boolean(loc: &Loc, value: bool) -> Self {
        Self::new(loc, if value { Val::True } else { Val::False })
    }

    pub fn singleton(loc: &Loc, tipo: &Type) -> Result<Self> {
        Ok(Self::new(
            loc,
            match tipo {
                Type::None => Ok(Val::None),
                Type::True => Ok(Val::True),
                Type::False => Ok(Val::False),
                Type::Singleton(_) => Ok(Val::Singleton(tipo.clone())),
                _ => loc.err(Error::SingletonExpected(tipo.clone())),
            }?,
        ))
    }

    pub fn borrow_type(&self) -> &Type {
        match &self.value {
            Val::None => &Type::None,
            Val::True => &Type::True,
            Val::False => &Type::False,
            Val::Integer(_) => &Type::Integer,
            Val::Singleton(t) => t,
        }
    }

    pub fn clone_type(&self) -> Type {
        self.borrow_type().clone()
    }

    pub fn as_integer(self) -> Option<Integer> {
        match self.value {
            Val::Integer(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_boolean(self) -> Option<bool> {
        match self.value {
            Val::True => Some(true),
            Val::False => Some(false),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Val::None => f.write_str("None"),
            Val::True => f.write_str("True"),
            Val::False => f.write_str("False"),
            Val::Integer(value) => value.fmt(f),
            Val::Singleton(t) => t.fmt(f),
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
        symbol: FQSym,
        visibility: Visibility,
        value: Value,
    ) -> Result<()> {
        if self.values.contains_key(&symbol) {
            value.loc.err(Error::DuplicateSymbol(symbol))
        } else {
            self.values.insert(symbol, Visible::new(visibility, value));
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueError {
    DuplicateValue(FQSym),
}
