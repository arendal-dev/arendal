use std::fmt;
use std::sync::Arc;

use im::HashMap;

use crate::error::{Error, Loc, Result};
use crate::symbol::FQType;
use crate::visibility::{Visibility, Visible};

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
pub enum Type {
    None,
    True,
    False,
    Boolean,
    Integer,
    Singleton(Singleton),
    Tuple(Arc<Tuple>),
}

#[derive(Clone, PartialEq, Eq)]
pub struct Tuple {
    symbol: FQType,
    types: Vec<FQType>,
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

#[derive(Debug, Clone)]
pub(crate) struct Types {
    types: HashMap<FQType, Visible<Type>>,
}

impl Default for Types {
    fn default() -> Self {
        let mut types: HashMap<FQType, Visible<Type>> = Default::default();
        types.insert(FQType::None, Visible::new(Visibility::Exported, Type::None));
        types.insert(FQType::True, Visible::new(Visibility::Exported, Type::True));
        types.insert(
            FQType::False,
            Visible::new(Visibility::Exported, Type::False),
        );
        types.insert(
            FQType::Boolean,
            Visible::new(Visibility::Exported, Type::Boolean),
        );
        types.insert(
            FQType::Integer,
            Visible::new(Visibility::Exported, Type::Integer),
        );
        Types { types }
    }
}

impl Types {
    pub(crate) fn get(&self, symbol: &FQType) -> Option<&Visible<Type>> {
        self.types.get(symbol)
    }

    pub fn singleton(&self, loc: &Loc, symbol: FQType) -> Result<Type> {
        if self.types.contains_key(&symbol) {
            loc.err(Error::DuplicateType(symbol))
        } else {
            let tipo = Type::Singleton(Singleton {
                symbol: symbol.clone(),
            });
            Ok(tipo)
        }
    }
}
