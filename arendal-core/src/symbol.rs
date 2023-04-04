use std::fmt::{self, Display, Write};
use std::sync::Arc;

use crate::error::{Error, Errors, Loc, Result};
use crate::keyword::Keyword;
use crate::{literal, ArcStr};

static STD: ArcStr = literal!("std");
static PKG: ArcStr = literal!("pkg");
pub(crate) static NONE: ArcStr = literal!("None");
pub(crate) static TRUE: ArcStr = literal!("True");
pub(crate) static FALSE: ArcStr = literal!("False");
pub(crate) static BOOLEAN: ArcStr = literal!("Boolean");
pub(crate) static INTEGER: ArcStr = literal!("Integer");

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

pub trait Sym: fmt::Display + fmt::Debug {}

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

impl Sym for Symbol {}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TSymbol {
    name: ArcStr,
}

impl TSymbol {
    fn known(name: &ArcStr) -> Result<Self> {
        Ok(TSymbol { name: name.clone() })
    }

    pub fn new(loc: Loc, name: ArcStr) -> Result<Self> {
        if name.is_empty() {
            return Errors::err(loc, SymbolError::Empty);
        }
        if NONE == name {
            Self::known(&NONE)
        } else if TRUE == name {
            Self::known(&TRUE)
        } else if FALSE == name {
            Self::known(&FALSE)
        } else if BOOLEAN == name {
            Self::known(&BOOLEAN)
        } else if INTEGER == name {
            Self::known(&INTEGER)
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
            Ok(TSymbol { name })
        }
    }

    pub(crate) fn is_none(&self) -> bool {
        NONE == self.name
    }

    pub(crate) fn is_true(&self) -> bool {
        TRUE == self.name
    }

    pub(crate) fn is_false(&self) -> bool {
        FALSE == self.name
    }

    pub(crate) fn is_boolean(&self) -> bool {
        BOOLEAN == self.name
    }

    pub(crate) fn is_integer(&self) -> bool {
        INTEGER == self.name
    }

    pub(crate) fn is_well_known(&self) -> bool {
        self.is_none()
            || self.is_true()
            || self.is_false()
            || self.is_boolean()
            || self.is_integer()
    }
}

impl fmt::Display for TSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

impl fmt::Debug for TSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "TSymbol", self)
    }
}

impl Sym for TSymbol {}

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
pub struct FQ<T: Sym> {
    pkg: PkgId,
    path: ModulePath,
    memberOf: Option<TSymbol>,
    symbol: T,
}

impl<T: Sym> FQ<T> {
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

impl<T: Sym> fmt::Display for FQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pkg.fmt(f)?;
        add_segment(f, &self.path)?;
        if let Some(m) = &self.memberOf {
            add_segment(f, m)?;
        }
        add_segment(f, &self.symbol)
    }
}

impl<T: Sym> fmt::Debug for FQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FQ<TSymbol> {
    pub(crate) fn is_none(&self) -> bool {
        self.symbol.is_none()
    }

    pub(crate) fn is_true(&self) -> bool {
        self.symbol.is_true()
    }

    pub(crate) fn is_false(&self) -> bool {
        self.symbol.is_false()
    }

    pub(crate) fn is_boolean(&self) -> bool {
        self.symbol.is_boolean()
    }

    pub(crate) fn is_integer(&self) -> bool {
        self.symbol.is_integer()
    }

    pub(crate) fn is_well_known(&self) -> bool {
        self.symbol.is_well_known()
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
