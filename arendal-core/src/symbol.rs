use phf::phf_map;
use std::fmt::{self, Display, Write};
use std::sync::Arc;

use crate::error::{Error, Loc, Result};
use crate::id::Id;
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
            return Error::err(loc, SymbolError::Empty);
        }
        if let Some(k) = Keyword::parse(name.as_str()) {
            return Error::err(loc, SymbolError::Keyword(k));
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_lowercase() {
                    return Error::err(loc, SymbolError::InvalidInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return Error::err(loc, SymbolError::InvalidChar(i, c));
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
            return Error::err(loc, SymbolError::Empty);
        }
        if let Some(s) = T_SYMBOLS.get(&name) {
            Ok(s.clone())
        } else {
            for (i, c) in name.char_indices() {
                if i == 0 {
                    if !c.is_ascii_alphabetic() || !c.is_ascii_uppercase() {
                        return Error::err(loc, SymbolError::InvalidTypeInitial(c));
                    }
                } else {
                    if !c.is_ascii_alphanumeric() {
                        return Error::err(loc, SymbolError::InvalidChar(i, c));
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
pub enum Pkg {
    Std,
    Local,
    External(Id),
}

impl Pkg {
    pub fn empty(&self) -> Path {
        Path::new(self.clone(), Vec::default())
    }

    pub fn single(&self, symbol: Symbol) -> Path {
        Path::new(self.clone(), vec![symbol])
    }

    pub fn path(&self, mut symbols: Vec<Symbol>) -> Path {
        match symbols.len() {
            0 => self.empty(),
            1 => self.single(symbols.pop().unwrap()),
            _ => Path::new(self.clone(), symbols),
        }
    }
}

impl fmt::Display for Pkg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Std => f.write_str(&STD),
            Self::Local => f.write_str(&PKG),
            Self::External(id) => write!(f, "pkg({})", id),
        }
    }
}

impl fmt::Debug for Pkg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "Pkg", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Path {
    pkg: Pkg,
    path: Arc<Vec<Symbol>>,
}

impl Path {
    fn new(pkg: Pkg, path: Vec<Symbol>) -> Self {
        Path {
            pkg,
            path: Arc::new(path),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    pub fn fq_sym(&self, symbol: Symbol) -> FQSym {
        FQSym::TopLevel(TopLevel {
            data: Arc::new(TLData {
                path: self.clone(),
                symbol,
            }),
        })
    }

    pub fn fq_type(&self, symbol: TSymbol) -> FQType {
        FQType::top_level(self.clone(), symbol)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pkg.fmt(f)?;
        for s in self.path.iter() {
            s.fmt(f)?
        }
        Ok(())
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "Path", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct TLData<T> {
    path: Path,
    symbol: T,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TopLevel<T> {
    data: Arc<TLData<T>>,
}

impl<T: Display> fmt::Display for TopLevel<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.path.fmt(f)?;
        add_segment(f, &self.data.symbol)
    }
}

impl<T: Display> fmt::Debug for TopLevel<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct MemberData<T> {
    top_level: FQType,
    symbol: T,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Member<T> {
    data: Arc<MemberData<T>>,
}

impl<T: Display> fmt::Display for Member<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.top_level.fmt(f)?;
        add_segment(f, &self.data.symbol)
    }
}

impl<T: Display> fmt::Debug for Member<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum FQSym {
    TopLevel(TopLevel<Symbol>),
    Member(Member<Symbol>),
}

impl FQSym {
    pub(crate) fn is_top_level(&self) -> bool {
        match self {
            Self::Member(_) => false,
            _ => true,
        }
    }

    pub fn symbol(&self) -> Symbol {
        match self {
            Self::TopLevel(t) => t.data.symbol.clone(),
            Self::Member(m) => m.data.symbol.clone(),
        }
    }
}

impl fmt::Display for FQSym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TopLevel(t) => t.fmt(f),
            Self::Member(m) => m.fmt(f),
        }
    }
}

impl fmt::Debug for FQSym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum FQType {
    None,
    True,
    False,
    Boolean,
    Integer,
    TopLevel(TopLevel<TSymbol>),
    Member(Member<TSymbol>),
}

impl FQType {
    fn get_known(path: &Path, symbol: &TSymbol) -> Option<Self> {
        if path.pkg == Pkg::Std && path.is_empty() {
            match symbol {
                TSymbol::None => Some(Self::None),
                TSymbol::True => Some(Self::True),
                TSymbol::False => Some(Self::False),
                TSymbol::Boolean => Some(Self::Boolean),
                TSymbol::Integer => Some(Self::Integer),
                _ => None,
            }
        } else {
            None
        }
    }

    pub(crate) fn is_known(&self) -> bool {
        match self {
            Self::TopLevel(_) | &Self::Member(_) => false,
            _ => true,
        }
    }

    fn top_level(path: Path, symbol: TSymbol) -> Self {
        if let Some(fq) = Self::get_known(&path, &symbol) {
            fq
        } else {
            Self::TopLevel(TopLevel {
                data: Arc::new(TLData { path, symbol }),
            })
        }
    }

    pub fn is_top_level(&self) -> bool {
        match self {
            Self::Member(_) => false,
            _ => true,
        }
    }

    pub(crate) fn member_sym(&self, loc: Loc, symbol: Symbol) -> Result<FQSym> {
        if self.is_top_level() {
            Ok(FQSym::Member(Member {
                data: Arc::new(MemberData {
                    top_level: self.clone(),
                    symbol,
                }),
            }))
        } else {
            Error::err(loc, SymbolError::ExpectedTopLevelType(self.clone()))
        }
    }

    pub(crate) fn member_type(&self, loc: Loc, symbol: TSymbol) -> Result<Self> {
        if self.is_top_level() {
            Ok(Self::Member(Member {
                data: Arc::new(MemberData {
                    top_level: self.clone(),
                    symbol,
                }),
            }))
        } else {
            Error::err(loc, SymbolError::ExpectedTopLevelType(self.clone()))
        }
    }

    pub fn symbol(&self) -> TSymbol {
        match self {
            Self::None => TSymbol::None,
            Self::True => TSymbol::True,
            Self::False => TSymbol::False,
            Self::Boolean => TSymbol::Boolean,
            Self::Integer => TSymbol::Integer,
            Self::TopLevel(t) => t.data.symbol.clone(),
            Self::Member(m) => m.data.symbol.clone(),
        }
    }
}

impl fmt::Display for FQType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("std::None"),
            Self::True => f.write_str("std::True"),
            Self::False => f.write_str("std::False"),
            Self::Boolean => f.write_str("std::Boolean"),
            Self::Integer => f.write_str("std::Integer"),
            Self::TopLevel(t) => t.fmt(f),
            Self::Member(m) => m.fmt(f),
        }
    }
}

impl fmt::Debug for FQType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolError {
    Empty,
    TypeEmpty,
    Keyword(Keyword),
    InvalidInitial(char),
    InvalidTypeInitial(char),
    InvalidChar(usize, char),
    ExpectedTypeItem(Symbol),
    ExpectedNonTypeItem(Symbol),
    ExpectedTopLevelType(FQType),
}
