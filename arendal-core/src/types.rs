use std::fmt;

use im::HashMap;

use crate::error::{Error, Errors, Loc, Result};
use crate::symbol::{FQType, TSymbol};
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
    fn ok_singleton(symbol: FQType) -> Result<Self> {
        Ok(Self::Singleton(Singleton { symbol }))
    }

    pub fn singleton(loc: Loc, symbol: FQType) -> Result<Self> {
        if symbol.is_known() {
            match symbol.symbol() {
                TSymbol::None => Ok(Self::None),
                TSymbol::True => Ok(Self::True),
                TSymbol::False => Ok(Self::False),
                _ => Errors::err(loc, TypeError::InvalidSingleton(symbol)),
            }
        } else {
            Self::ok_singleton(symbol)
        }
    }

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

#[derive(Debug, Default)]
pub(crate) struct Types {
    types: HashMap<FQType, Visible<Type>>,
}

impl Types {
    pub(crate) fn add(&mut self, loc: Loc, visibility: Visibility, tipo: Type) -> Result<()> {
        let fq = tipo.fq();
        if self.types.contains_key(&fq) {
            Errors::err(loc, TypeError::DuplicateType(tipo))
        } else {
            self.types.insert(fq, Visible::new(visibility, tipo));
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum TypeError {
    InvalidSingleton(FQType),
    DuplicateType(Type),
}

impl Error for TypeError {}
