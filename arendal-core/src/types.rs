use std::fmt;

use crate::error::{Error, Errors, Loc, Result};
use crate::symbol::{ModulePath, PkgId, TSymbol, FQ};

#[derive(Clone, PartialEq, Eq)]
pub struct Singleton {
    symbol: FQ<TSymbol>,
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
    fn ok_singleton(symbol: FQ<TSymbol>) -> Result<Self> {
        Ok(Self::Singleton(Singleton { symbol }))
    }

    pub fn singleton(loc: Loc, symbol: FQ<TSymbol>) -> Result<Self> {
        if symbol.is_std() {
            match symbol.symbol() {
                TSymbol::None => Ok(Self::None),
                TSymbol::True => Ok(Self::True),
                TSymbol::False => Ok(Self::False),
                TSymbol::Other(_) => Self::ok_singleton(symbol),
                _ => Errors::err(loc, TypeConstructionError::InvalidSingleton(symbol)),
            }
        } else {
            Self::ok_singleton(symbol)
        }
    }

    fn fq_std(symbol: TSymbol) -> FQ<TSymbol> {
        FQ::top_level(PkgId::std(), ModulePath::empty(), symbol)
    }

    pub fn fq(&self) -> FQ<TSymbol> {
        match self {
            Self::None => Self::fq_std(TSymbol::None),
            Self::True => Self::fq_std(TSymbol::None),
            Self::False => Self::fq_std(TSymbol::None),
            Self::Boolean => Self::fq_std(TSymbol::None),
            Self::Integer => Self::fq_std(TSymbol::None),
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

#[derive(Debug)]
pub enum TypeConstructionError {
    InvalidSingleton(FQ<TSymbol>),
}

impl Error for TypeConstructionError {}
