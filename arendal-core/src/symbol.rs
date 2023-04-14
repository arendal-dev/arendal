use phf::phf_map;
use std::fmt::{self, Display, Write};
use std::sync::Arc;

use crate::error::{Error, Errors, Loc, Result};
use crate::keyword::Keyword;
use crate::{literal, ArcStr};

static STD: ArcStr = literal!("std");
static PKG: ArcStr = literal!("pkg");

fn separator(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("::")
}

fn add_segment(f: &mut fmt::Formatter<'_>, it: &dyn Display) -> fmt::Result {
    separator(f)?;
    it.fmt(f)
}

fn debug(f: &mut fmt::Formatter<'_>, name: &str, it: &dyn fmt::Display) -> fmt::Result {
    f.write_str(name)?;
    f.write_char('(')?;
    it.fmt(f)?;
    f.write_char(')')
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PkgId {
    id: u32,
}

impl PkgId {
    pub fn new(id: u32) -> Self {
        PkgId { id }
    }

    pub fn std() -> Self {
        Self::new(0)
    }

    pub fn local() -> Self {
        Self::new(1)
    }
}

impl fmt::Display for PkgId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.id {
            0 => f.write_str(&STD),
            1 => f.write_str(&PKG),
            n => write!(f, "pkg({})", n),
        }
    }
}

impl fmt::Debug for PkgId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "PkgId", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Sym<T> {
    Known(T),
    Other(ArcStr),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Other {
    name: ArcStr,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    name: ArcStr,
}

impl Symbol {
    pub fn new(loc: Loc, name: ArcStr) -> Result<Self> {
        if name.is_empty() {
            return Errors::err(loc, SymbolError::Empty);
        }
        if let Some(k) = Keyword::parse(name.as_str()) {
            return Errors::err(loc, SymbolError::Keyword(k));
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_lowercase() {
                    return Errors::err(loc, SymbolError::InvalidInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return Errors::err(loc, SymbolError::InvalidChar(i, c));
                }
            }
        }
        Ok(Self { name: name.into() })
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "Symbol", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TSymbol {
    None,
    True,
    False,
    Boolean,
    Integer,
    Other(Other),
}

static T_SYMBOLS: phf::Map<&'static str, TSymbol> = phf_map! {
    "None" => TSymbol::None,
    "True" => TSymbol::True,
    "False" => TSymbol::False,
    "Boolean" => TSymbol::Boolean,
    "Integer" => TSymbol::Integer,
};

impl TSymbol {
    pub fn new(loc: Loc, name: ArcStr) -> Result<Self> {
        if name.is_empty() {
            return Errors::err(loc, SymbolError::Empty);
        }
        if let Some(s) = T_SYMBOLS.get(&name) {
            Ok(s.clone())
        } else {
            for (i, c) in name.char_indices() {
                if i == 0 {
                    if !c.is_ascii_alphabetic() || !c.is_ascii_uppercase() {
                        return Errors::err(loc, SymbolError::InvalidTypeInitial(c));
                    }
                } else {
                    if !c.is_ascii_alphanumeric() {
                        return Errors::err(loc, SymbolError::InvalidChar(i, c));
                    }
                }
            }
            Ok(Self::Other(Other { name }))
        }
    }

    pub(crate) fn is_known(&self) -> bool {
        !matches!(self, Self::Other(_))
    }
}

impl fmt::Display for TSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("None"),
            Self::True => f.write_str("True"),
            Self::False => f.write_str("False"),
            Self::Boolean => f.write_str("Boolean"),
            Self::Integer => f.write_str("Integer"),
            Self::Other(o) => f.write_str(&o.name),
        }
    }
}

impl fmt::Debug for TSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "TSymbol", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModulePath {
    path: Arc<Vec<Symbol>>,
}

impl ModulePath {
    pub(crate) fn new(path: Vec<Symbol>) -> Self {
        ModulePath {
            path: Arc::new(path),
        }
    }

    pub(crate) fn empty() -> Self {
        Self::new(Default::default())
    }

    pub(crate) fn single(symbol: Symbol) -> Self {
        Self::new(vec![symbol])
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, id) in self.path.iter().enumerate() {
            if i > 0 {
                separator(f)?
            }
            id.fmt(f)?
        }
        Ok(())
    }
}

impl fmt::Debug for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "ModulePath", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FQ<T> {
    pkg: PkgId,
    path: ModulePath,
    memberOf: Option<TSymbol>,
    symbol: T,
}

impl<T> FQ<T> {
    pub(crate) fn is_std(&self) -> bool {
        0 == self.pkg.id && self.path.is_empty()
    }

    pub(crate) fn top_level(pkg: PkgId, path: ModulePath, symbol: T) -> Self {
        FQ {
            pkg,
            path,
            memberOf: None,
            symbol,
        }
    }

    pub(crate) fn member(pkg: PkgId, path: ModulePath, memberOf: TSymbol, symbol: T) -> Self {
        FQ {
            pkg,
            path,
            memberOf: Some(memberOf),
            symbol,
        }
    }

    fn with_pkg(self, pkg: PkgId) -> Self {
        if pkg == self.pkg {
            self
        } else {
            FQ {
                pkg,
                path: self.path,
                memberOf: self.memberOf,
                symbol: self.symbol,
            }
        }
    }
}

impl<T: Display> fmt::Display for FQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pkg.fmt(f)?;
        add_segment(f, &self.path)?;
        if let Some(m) = &self.memberOf {
            add_segment(f, m)?;
        }
        add_segment(f, &self.symbol)
    }
}

impl<T: Display> fmt::Debug for FQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FQ<TSymbol> {
    pub(crate) fn is_none(&self) -> bool {
        matches!(self.symbol, TSymbol::None)
    }

    pub(crate) fn is_true(&self) -> bool {
        matches!(self.symbol, TSymbol::True)
    }

    pub(crate) fn is_false(&self) -> bool {
        matches!(self.symbol, TSymbol::False)
    }

    pub(crate) fn is_boolean(&self) -> bool {
        matches!(self.symbol, TSymbol::Boolean)
    }

    pub(crate) fn is_integer(&self) -> bool {
        matches!(self.symbol, TSymbol::Integer)
    }

    pub(crate) fn is_well_known(&self) -> bool {
        !matches!(self.symbol, TSymbol::Other(_))
    }
}

#[derive(Debug)]
pub enum SymbolError {
    Empty,
    TypeEmpty,
    Keyword(Keyword),
    InvalidInitial(char),
    InvalidTypeInitial(char),
    InvalidChar(usize, char),
    ExpectedTypeItem(Symbol),
    ExpectedNonTypeItem(Symbol),
}

impl Error for SymbolError {}
