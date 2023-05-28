mod prelude;
mod tst;

use im::HashMap;

use crate::{
    error::{Error, Loc, Result},
    symbol::{FQSym, FQType},
    types::Type,
    visibility::{Visibility, V},
    Integer,
};

use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct ValidType {
    tipo: Type,
}

impl fmt::Display for ValidType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tipo.fmt(f)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    None,
    True,
    False,
    Integer(Integer),
    Singleton(ValidType),
}

impl Value {
    pub fn boolean(value: bool) -> Self {
        if value {
            Value::True
        } else {
            Value::False
        }
    }

    pub fn singleton(loc: &Loc, tipo: &Type) -> Result<Self> {
        match tipo {
            Type::None => Ok(Value::None),
            Type::True => Ok(Value::True),
            Type::False => Ok(Value::False),
            Type::Singleton(_) => Ok(Value::Singleton(ValidType { tipo: tipo.clone() })),
            _ => loc.err(Error::SingletonExpected(tipo.clone())),
        }
    }

    pub fn borrow_type(&self) -> &Type {
        match self {
            Value::None => &Type::None,
            Value::True => &Type::True,
            Value::False => &Type::False,
            Value::Integer(_) => &Type::Integer,
            Value::Singleton(t) => &t.tipo,
        }
    }

    pub fn clone_type(&self) -> Type {
        self.borrow_type().clone()
    }

    pub fn as_integer(self, loc: &Loc) -> Result<Integer> {
        match self {
            Value::Integer(v) => Ok(v),
            _ => self.type_mismatch(loc, Type::Integer),
        }
    }

    pub fn as_boolean(self, loc: &Loc) -> Result<bool> {
        match self {
            Value::True => Ok(true),
            Value::False => Ok(false),
            _ => self.type_mismatch(loc, Type::Boolean),
        }
    }

    fn type_mismatch<T>(&self, loc: &Loc, expected: Type) -> Result<T> {
        loc.err(Error::type_mismatch(expected, self.clone_type()))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => f.write_str("None"),
            Value::True => f.write_str("True"),
            Value::False => f.write_str("False"),
            Value::Integer(value) => value.fmt(f),
            Value::Singleton(t) => t.fmt(f),
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
    values: HashMap<FQSym, V<Value>>,
}

impl Values {
    pub(crate) fn get(&self, symbol: &FQSym) -> Option<V<Value>> {
        self.values.get(symbol).cloned()
    }

    pub(crate) fn set(
        &mut self,
        loc: &Loc,
        symbol: FQSym,
        visibility: Visibility,
        value: Value,
    ) -> Result<()> {
        if self.values.contains_key(&symbol) {
            loc.err(Error::DuplicateSymbol(symbol))
        } else {
            self.values.insert(symbol, visibility.wrap(value));
            Ok(())
        }
    }
}

type TypeMap = HashMap<FQType, V<Type>>;

#[derive(Debug, Clone)]
struct Types {
    types: TypeMap,
}

impl Default for Types {
    fn default() -> Self {
        Types {
            types: prelude::load_types(),
        }
    }
}

impl Types {
    pub(crate) fn get(&self, symbol: &FQType) -> Option<&V<Type>> {
        self.types.get(symbol)
    }

    fn contains(&self, symbol: &FQType) -> bool {
        self.types.contains_key(symbol)
    }

    // temporary
    pub fn singleton(&self, loc: &Loc, symbol: FQType) -> Result<Type> {
        if self.types.contains_key(&symbol) {
            loc.err(Error::DuplicateType(symbol))
        } else {
            Type::singleton(loc, symbol)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Env {
    types: Types,
    values: Values,
}

impl Env {
    pub fn run(&mut self, input: &str) -> Result<Value> {
        tst::run(self, input)
    }
}
