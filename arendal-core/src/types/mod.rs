mod builtin;

use std::fmt;
use std::sync::Arc;

use im::HashMap;

use crate::error::{Error, Loc, Result};
use crate::symbol::FQType;
use crate::visibility::V;

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    None,
    True,
    False,
    Boolean,
    Integer,
    Singleton(Singleton),
    Tuple(Arc<Tuple>),
}

// We need some kind of indirection to reference types
// to allow for recursive type definitions.
#[derive(Clone, PartialEq, Eq)]
enum Ref {
    Type(Type),
    Symbol(FQType),
}

#[derive(Clone, PartialEq, Eq)]
pub struct Singleton {
    symbol: FQType,
}

impl fmt::Display for Singleton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.symbol.fmt(f)
    }
}

impl fmt::Debug for Singleton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Tuple {
    symbol: FQType,
    types: Vec<Ref>,
}

impl Type {
    pub fn fq(&self) -> FQType {
        match self {
            Self::None => FQType::None,
            Self::True => FQType::True,
            Self::False => FQType::False,
            Self::Boolean => FQType::Boolean,
            Self::Integer => FQType::Integer,
            Self::Singleton(s) => s.symbol.clone(),
            Self::Tuple(t) => t.symbol.clone(),
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            Self::True | Self::False | Self::Boolean => true,
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        *self == Self::Integer
    }

    pub fn is_singleton(&self) -> bool {
        match self {
            Type::None | Type::True | Type::False | Type::Singleton(_) => true,
            _ => false,
        }
    }

    pub fn singleton(loc: &Loc, symbol: FQType) -> Result<Type> {
        if symbol.is_known() {
            loc.err(Error::DuplicateType(symbol))
        } else {
            let tipo = Type::Singleton(Singleton {
                symbol: symbol.clone(),
            });
            Ok(tipo)
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fq().fmt(f)
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

type TypeMap = HashMap<FQType, V<Type>>;

#[derive(Debug, Clone)]
pub struct Types {
    types: TypeMap,
}

impl Default for Types {
    fn default() -> Self {
        Types {
            types: builtin::get_builtin_types(),
        }
    }
}

impl Types {
    pub(crate) fn get(&self, symbol: &FQType) -> Option<&V<Type>> {
        self.types.get(symbol)
    }

    pub fn contains(&self, symbol: &FQType) -> bool {
        self.types.contains_key(symbol)
    }
}
