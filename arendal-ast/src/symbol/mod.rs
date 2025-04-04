use std::fmt::{self, Write};
use std::hash::Hash;
use std::sync::Arc;

use id::Id;
use crate::keyword::Keyword;
use arcstr::ArcStr;

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
    InvalidChar(usize, char)
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
    path: Arc<Vec<Symbol>>,
}

impl ModulePath {
    pub fn new(path: Vec<Symbol>) -> Self {
        Self { path: Arc::new(path) }
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
        FQPath { data: Arc::new(FQPathData { lib, path }) }
    }

    pub fn std_empty() -> FQPath {
        Self::new(Lib::Std, ModulePath::empty())
    }

    pub fn is_empty(&self) -> bool {
        self.data.path.is_empty()
    }

    pub fn fq_sym(&self, symbol: Symbol) -> FQSym {
        FQ::top_level(self.clone(), symbol)
    }

    pub fn fq_type(&self, symbol: TSymbol) -> FQType {
        FQ::top_level(self.clone(), symbol)
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
enum FQData<T> {
    TopLevel(FQPath, T),
    Enclosed(Arc<FQType>, T),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FQ<T> {
    data: FQData<T>,
}

impl<T> FQ<T> {
    fn top_level(path: FQPath, symbol: T) -> FQ<T> {
        FQ {
            data: FQData::TopLevel(path, symbol),
        }
    }

    pub fn is_top_level(&self) -> bool {
        matches!(self.data, FQData::TopLevel(_, _))
    }

    pub fn path(&self) -> FQPath {
        match &self.data {
            FQData::TopLevel(path, _) => path.clone(),
            FQData::Enclosed(parent, _) => parent.path(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for FQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            FQData::TopLevel(path, symbol) => {
                path.fmt(f)?;
                separator(f)?;
                symbol.fmt(f)
            }
            FQData::Enclosed(parent, symbol) => {
                parent.fmt(f)?;
                separator(f)?;
                symbol.fmt(f)
            }
        }
    }
}

impl<T: fmt::Display> fmt::Debug for FQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "FQ", self)
    }
}

pub type FQSym = FQ<Symbol>;
pub type FQType = FQ<TSymbol>;

