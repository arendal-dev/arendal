use std::fmt::{self, Write};
use std::sync::Arc;

use phf::phf_map;

use crate::error::{Error, Errors, Loc, Result};
use crate::id::Id;
use crate::keyword::Keyword;
use crate::{literal, ArcStr};

static STD: ArcStr = literal!("std");
static PKG: ArcStr = literal!("pkg");
static EMPTY: ArcStr = literal!("");

fn debug(f: &mut fmt::Formatter<'_>, name: &str, it: &dyn fmt::Display) -> fmt::Result {
    f.write_str(name)?;
    f.write_char('(')?;
    it.fmt(f)?;
    f.write_char(')')
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PkgId {
    Std,
    Local,
    Imported(Id),
}

impl PkgId {
    fn as_arcstr(&self) -> ArcStr {
        match self {
            PkgId::Std => STD.clone(),
            PkgId::Local => PKG.clone(),
            PkgId::Imported(id) => id.as_arcstr(),
        }
    }
}

impl fmt::Display for PkgId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_arcstr())
    }
}

impl fmt::Debug for PkgId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "PkgId", self)
    }
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

    fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "Symbol", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum InnerTSym {
    None,
    True,
    False,
    Boolean,
    Integer,
    Other(ArcStr),
}

static WELL_KNOWN: phf::Map<&'static str, InnerTSym> = phf_map! {
    "None" => InnerTSym::None,
    "True" => InnerTSym::True,
    "False" => InnerTSym::False,
    "Boolean" => InnerTSym::Boolean,
    "Integer" => InnerTSym::Integer,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TSymbol {
    inner: InnerTSym,
}

impl TSymbol {
    pub fn new(loc: Loc, name: ArcStr) -> Result<Self> {
        if name.is_empty() {
            return Errors::err(loc, SymbolError::Empty);
        }
        if let Some(s) = WELL_KNOWN.get(&name) {
            return Ok(Self { inner: s.clone() });
        }
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
        Ok(Self {
            inner: InnerTSym::Other(name),
        })
    }

    fn as_str(&self) -> &str {
        match &self.inner {
            InnerTSym::None => "None",
            InnerTSym::True => "True",
            InnerTSym::False => "False",
            InnerTSym::Boolean => "Boolean",
            InnerTSym::Integer => "Integer",
            InnerTSym::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for TSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Debug for TSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "TSymbol", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum InnerPath {
    Empty,
    Single(Symbol),
    Multi(Arc<Vec<Symbol>>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModulePath {
    path: InnerPath,
}

impl ModulePath {
    pub(crate) const fn empty() -> Self {
        ModulePath {
            path: InnerPath::Empty,
        }
    }

    pub(crate) const fn single(symbol: Symbol) -> Self {
        ModulePath {
            path: InnerPath::Single(symbol),
        }
    }

    pub(crate) fn new(mut path: Vec<Symbol>) -> Self {
        if path.is_empty() {
            Self::empty()
        } else if path.len() == 1 {
            Self::single(path.pop().unwrap())
        } else {
            ModulePath {
                path: InnerPath::Multi(Arc::new(path)),
            }
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        if let InnerPath::Empty = self.path {
            true
        } else {
            false
        }
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            InnerPath::Empty => Ok(()),
            InnerPath::Single(s) => s.fmt(f),
            InnerPath::Multi(path) => {
                for (i, id) in path.iter().enumerate() {
                    if i > 0 {
                        f.write_str("::")?
                    }
                    id.fmt(f)?
                }
                Ok(())
            }
        }
    }
}

impl fmt::Debug for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "ModulePath", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct TL<T> {
    pkg: PkgId,
    path: ModulePath,
    symbol: T,
}

impl<T: Clone> TL<T> {
    fn with_pkg(&self, pkg: PkgId) -> Self {
        TL {
            pkg,
            path: self.path.clone(),
            symbol: self.symbol.clone(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for TL<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pkg.fmt(f)?;
        if !self.path.is_empty() {
            self.path.fmt(f)?;
            f.write_str("::")?;
        }
        self.symbol.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Debug for TL<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum InnerFQ<T> {
    TopLevel(TL<T>),
    Member(TL<TSymbol>, T),
}

impl<T: Clone> InnerFQ<T> {
    fn with_pkg(&self, pkg: PkgId) -> Self {
        match self {
            Self::TopLevel(t) => Self::TopLevel(t.with_pkg(pkg)),
            Self::Member(t, s) => Self::Member(t.with_pkg(pkg), s.clone()),
        }
    }
}

impl<T: fmt::Display> fmt::Display for InnerFQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InnerFQ::TopLevel(t) => t.fmt(f),
            InnerFQ::Member(t, s) => {
                t.fmt(f)?;
                f.write_str("::")?;
                s.fmt(f)
            }
        }
    }
}

impl<T: fmt::Display> fmt::Debug for InnerFQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FQSymbol {
    inner: InnerFQ<Symbol>,
}

impl FQSymbol {
    pub fn top_level(pkg: PkgId, path: ModulePath, symbol: Symbol) -> Self {
        Self {
            inner: InnerFQ::TopLevel(TL { pkg, path, symbol }),
        }
    }

    pub fn std(symbol: Symbol) -> Self {
        Self::top_level(PkgId::Std, ModulePath::empty(), symbol)
    }

    pub fn member(pkg: PkgId, path: ModulePath, parent: TSymbol, symbol: Symbol) -> Self {
        Self {
            inner: InnerFQ::Member(
                TL {
                    pkg,
                    path,
                    symbol: parent,
                },
                symbol,
            ),
        }
    }

    pub(crate) fn with_pkg(&self, pkg: PkgId) -> Self {
        FQSymbol {
            inner: self.inner.with_pkg(pkg),
        }
    }
}

impl fmt::Display for FQSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Debug for FQSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "FQSymbol", &self.inner)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FQTSymbol {
    inner: InnerFQ<TSymbol>,
}

impl FQTSymbol {
    pub fn top_level(pkg: PkgId, path: ModulePath, symbol: TSymbol) -> Self {
        FQTSymbol {
            inner: InnerFQ::TopLevel(TL { pkg, path, symbol }),
        }
    }

    pub fn std(symbol: TSymbol) -> Self {
        Self::top_level(PkgId::Std, ModulePath::empty(), symbol)
    }

    pub fn member(pkg: PkgId, path: ModulePath, parent: TSymbol, symbol: TSymbol) -> Self {
        Self {
            inner: InnerFQ::Member(
                TL {
                    pkg,
                    path,
                    symbol: parent,
                },
                symbol,
            ),
        }
    }

    pub(crate) fn with_pkg(&self, pkg: PkgId) -> Self {
        FQTSymbol {
            inner: self.inner.with_pkg(pkg),
        }
    }
}

impl fmt::Display for FQTSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Debug for FQTSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "FQTSymbol", &self.inner)
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
