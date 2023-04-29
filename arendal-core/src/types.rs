use std::fmt;

use im::HashMap;

use crate::error::{Errors, Loc, Result};
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
        }
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
    pub fn get(&self, symbol: &FQType) -> Option<&Visible<Type>> {
        self.types.get(symbol)
    }

    pub fn singleton(&mut self, loc: Loc, visibility: Visibility, symbol: FQType) -> Result<Type> {
        if self.types.contains_key(&symbol) {
            Errors::err(loc, TypesError::DuplicateType(symbol))
        } else {
            let tipo = Type::Singleton(Singleton {
                symbol: symbol.clone(),
            });
            self.types
                .insert(symbol, Visible::new(visibility, tipo.clone()));
            Ok(tipo)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypesError {
    DuplicateType(FQType),
}
