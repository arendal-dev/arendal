use std::fmt;

use crate::error::{Error, Errors, Loc, Result};
use crate::symbol::{TSymbol, FQ};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherType {
    symbol: FQ<TSymbol>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    None,
    True,
    False,
    Boolean,
    Integer,
    Singleton(OtherType),
}

impl Type {
    fn ok_singleton(symbol: FQ<TSymbol>) -> Result<Self> {
        Ok(Self::Singleton(OtherType { symbol }))
    }

    pub fn singleton(loc: Loc, symbol: FQ<TSymbol>) -> Result<Self> {
        if symbol.is_std() {
            if symbol.is_none() {
                Ok(Self::None)
            } else if symbol.is_true() {
                Ok(Self::True)
            } else if symbol.is_false() {
                Ok(Self::False)
            } else if symbol.is_well_known() {
                Errors::err(loc, TypeConstructionError::InvalidSingleton(symbol))
            } else {
                Self::ok_singleton(symbol)
            }
        } else {
            Self::ok_singleton(symbol)
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("std::None"),
            Self::True => f.write_str("std::True"),
            Self::False => f.write_str("std::False"),
            Self::Boolean => f.write_str("std::Boolean"),
            Self::Integer => f.write_str("std::Integer"),
            Self::Singleton(t) => t.symbol.fmt(f),
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[derive(Debug)]
pub enum TypeConstructionError {
    InvalidSingleton(FQ<TSymbol>),
}

impl Error for TypeConstructionError {}
