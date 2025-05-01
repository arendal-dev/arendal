use std::fmt::{self, Write};
use std::hash::Hash;
use std::sync::Arc;

use crate::keyword::Keyword;
use arcstr::ArcStr;
use id::Id;

fn separator(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("::")
}

fn add_segment(f: &mut fmt::Formatter<'_>, it: &dyn fmt::Display) -> fmt::Result {
    separator(f)?;
    it.fmt(f)
}

fn debug(f: &mut fmt::Formatter<'_>, name: &str, it: &dyn fmt::Display) -> fmt::Result {
    f.write_str(name)?;
    f.write_char('(')?;
    it.fmt(f)?;
    f.write_char(')')
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Other {
    name: ArcStr,
}

#[derive(Debug)]
pub enum Error {
    Empty,
    Keyword(Keyword),
    InvalidInitial(char),
    InvalidChar(usize, char),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    name: ArcStr,
}

impl Symbol {
    pub fn new(name: &str) -> Result<Self, Error> {
        if name.is_empty() {
            return Err(Error::Empty);
        }
        if let Some(k) = Keyword::parse(name) {
            return Err(Error::Keyword(k));
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_lowercase() {
                    return Err(Error::InvalidInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return Err(Error::InvalidChar(i, c));
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
pub struct TSymbol {
    name: ArcStr,
}

impl TSymbol {
    pub fn new(name: &str) -> Result<Self, Error> {
        if name.is_empty() {
            return Err(Error::Empty);
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_uppercase() {
                    return Err(Error::InvalidInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return Err(Error::InvalidChar(i, c));
                }
            }
        }
        Ok(Self { name: name.into() })
    }
}

impl fmt::Display for TSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl fmt::Debug for TSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "TSymbol", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Lib {
    Std,
    Local,
    External(Id),
}

impl Lib {
    pub fn path(&self, path: ModulePath) -> FQPath {
        FQPath::new(self.clone(), path)
    }

    pub fn empty(&self) -> FQPath {
        self.path(ModulePath::empty())
    }
}

impl fmt::Display for Lib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Std => f.write_str("std"),
            Self::Local => Ok(()),
            Self::External(id) => write!(f, "lib({})", id),
        }
    }
}

impl fmt::Debug for Lib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "Lib", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModulePath {
    path: Vec<Symbol>,
}

impl ModulePath {
    pub fn new(path: Vec<Symbol>) -> Self {
        Self { path }
    }

    pub fn empty() -> Self {
        Self::new(Vec::default())
    }

    pub fn single(symbol: Symbol) -> Self {
        Self::new(vec![symbol])
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.path.len();
        if len > 0 {
            for (i, s) in self.path.iter().enumerate() {
                s.fmt(f)?;
                if i < (len - 1) {
                    separator(f)?;
                }
            }
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
struct FQPathData {
    lib: Lib,
    path: ModulePath,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FQPath {
    data: Arc<FQPathData>,
}

impl FQPath {
    pub fn new(lib: Lib, path: ModulePath) -> Self {
        FQPath {
            data: Arc::new(FQPathData { lib, path }),
        }
    }

    pub fn std_empty() -> FQPath {
        Self::new(Lib::Std, ModulePath::empty())
    }

    pub fn is_empty(&self) -> bool {
        self.data.path.is_empty()
    }

    pub fn fq_sym(&self, symbol: Symbol) -> FQSym {
        FQSym::top_level(self.clone(), symbol)
    }

    pub fn fq_type(&self, symbol: TSymbol) -> FQType {
        FQType::top_level(self.clone(), symbol)
    }
}

impl fmt::Display for FQPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.lib.fmt(f)?;
        self.data.path.fmt(f)
    }
}

impl fmt::Debug for FQPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "FQPath", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TopLevel<T> {
    path: FQPath,
    symbol: T,
}

impl<T: fmt::Display> fmt::Display for TopLevel<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.path.fmt(f)?;
        separator(f)?;
        self.symbol.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Debug for TopLevel<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "TopLevel", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Nested<T> {
    parent: Arc<FQType>,
    symbol: T,
}

impl<T: fmt::Display> fmt::Display for Nested<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.parent.fmt(f)?;
        separator(f)?;
        self.symbol.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Debug for Nested<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "Nested", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum FQSym {
    Local(Symbol),
    TopLevel(TopLevel<Symbol>),
    Nested(Nested<Symbol>),
}

impl FQSym {
    fn top_level(path: FQPath, symbol: Symbol) -> Self {
        FQSym::TopLevel(TopLevel { path, symbol })
    }

    pub fn is_top_level(&self) -> bool {
        matches!(self, FQSym::TopLevel(_))
    }

    pub fn path(&self) -> Option<FQPath> {
        match &self {
            FQSym::Local(_) => None,
            FQSym::TopLevel(tl) => Some(tl.path.clone()),
            FQSym::Nested(n) => n.parent.path(),
        }
    }
}

impl fmt::Display for FQSym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            FQSym::Local(s) => s.fmt(f),
            FQSym::TopLevel(tl) => tl.fmt(f),
            FQSym::Nested(n) => n.fmt(f),
        }
    }
}

impl fmt::Debug for FQSym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "FQSym", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum FQType {
    Unit,
    TopLevel(TopLevel<TSymbol>),
    Nested(Nested<TSymbol>),
}

impl FQType {
    fn top_level(path: FQPath, symbol: TSymbol) -> Self {
        FQType::TopLevel(TopLevel { path, symbol })
    }

    pub fn is_top_level(&self) -> bool {
        matches!(self, FQType::TopLevel(_))
    }

    pub fn path(&self) -> Option<FQPath> {
        match &self {
            FQType::Unit => None,
            FQType::TopLevel(tl) => Some(tl.path.clone()),
            FQType::Nested(n) => n.parent.path(),
        }
    }
}

impl fmt::Display for FQType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            FQType::Unit => write!(f, "()"),
            FQType::TopLevel(tl) => tl.fmt(f),
            FQType::Nested(n) => n.fmt(f),
        }
    }
}

impl fmt::Debug for FQType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "FQType", self)
    }
}
