use phf::phf_map;
use std::fmt::{self, Display, Write};
use std::sync::Arc;

use crate::error::{Error, Loc, Result};
use crate::id::Id;
use crate::keyword::Keyword;
use crate::visibility::Visibility;
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
enum Sym<T> {
    One(char),
    Known(T),
    Other(ArcStr),
}

impl<T: fmt::Display> fmt::Display for Sym<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One(c) => f.write_char(*c),
            Self::Known(s) => s.fmt(f),
            Self::Other(s) => f.write_str(&s),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    name: ArcStr,
}

impl Symbol {
    pub fn new(loc: &Loc, name: ArcStr) -> Result<Self> {
        if name.is_empty() {
            return loc.err(Error::SymbolEmpty);
        }
        if let Some(k) = Keyword::parse(name.as_str()) {
            return loc.err(Error::SymbolKeywordFound(k));
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_lowercase() {
                    return loc.err(Error::SymbolInvalidInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return loc.err(Error::SymbolInvalidChar(i, c));
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum KnownTSymbol {
    None,
    True,
    False,
    Boolean,
    Integer,
}

impl fmt::Display for KnownTSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("None"),
            Self::True => f.write_str("True"),
            Self::False => f.write_str("False"),
            Self::Boolean => f.write_str("Boolean"),
            Self::Integer => f.write_str("Integer"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TSymbol {
    symbol: Sym<KnownTSymbol>,
}

static T_SYMBOLS: phf::Map<&'static str, TSymbol> = phf_map! {
    "None" => TSymbol::known(KnownTSymbol::None),
    "True" => TSymbol::known(KnownTSymbol::True),
    "False" => TSymbol::known(KnownTSymbol::False),
    "Boolean" => TSymbol::known(KnownTSymbol::Boolean),
    "Integer" => TSymbol::known(KnownTSymbol::Integer),
};

impl TSymbol {
    const fn known(symbol: KnownTSymbol) -> Self {
        Self {
            symbol: Sym::Known(symbol),
        }
    }

    pub fn new(loc: &Loc, name: ArcStr) -> Result<Self> {
        if name.is_empty() {
            return loc.err(Error::TSymbolEmpty);
        }
        if let Some(s) = T_SYMBOLS.get(&name) {
            Ok(s.clone())
        } else {
            for (i, c) in name.char_indices() {
                if i == 0 {
                    if !c.is_ascii_alphabetic() || !c.is_ascii_uppercase() {
                        return loc.err(Error::TSymbolInvalidInitial(c));
                    }
                } else {
                    if !c.is_ascii_alphanumeric() {
                        return loc.err(Error::SymbolInvalidChar(i, c));
                    }
                }
            }
            Ok(Self {
                symbol: Sym::Other(name),
            })
        }
    }

    pub(crate) fn is_known(&self) -> bool {
        !matches!(self.symbol, Sym::Known(_))
    }
}

impl fmt::Display for TSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.symbol.fmt(f)
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
    pub fn path(&self, path: Path) -> FQPath {
        FQPath {
            pkg: self.clone(),
            path,
        }
    }

    pub fn empty(&self) -> FQPath {
        self.path(Path::empty())
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
enum PathData {
    Empty,
    Single(Symbol),
    Multi(Arc<Vec<Symbol>>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Path {
    data: PathData,
}

impl Path {
    pub fn new(mut path: Vec<Symbol>) -> Self {
        match path.len() {
            0 => Self::empty(),
            1 => Self::single(path.pop().unwrap()),
            _ => Self {
                data: PathData::Multi(Arc::new(path)),
            },
        }
    }

    pub const fn empty() -> Self {
        Self {
            data: PathData::Empty,
        }
    }

    pub const fn single(symbol: Symbol) -> Self {
        Self {
            data: PathData::Single(symbol),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data == PathData::Empty
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            PathData::Empty => Ok(()),
            PathData::Single(s) => s.fmt(f),
            PathData::Multi(v) => {
                for (i, s) in v.iter().enumerate() {
                    s.fmt(f)?;
                    if i < (v.len() - 1) {
                        separator(f)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "Path", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FQPath {
    pub pkg: Pkg,
    pub path: Path,
}

impl FQPath {
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    pub fn fq_sym(&self, symbol: Symbol) -> FQSym {
        FQ {
            path: self.clone(),
            enclosing: None,
            symbol,
        }
    }

    pub fn fq_type(&self, symbol: TSymbol) -> FQType {
        FQ {
            path: self.clone(),
            enclosing: None,
            symbol,
        }
    }

    pub fn can_see(&self, visibility: Visibility, path: &FQPath) -> bool {
        match visibility {
            Visibility::Exported => true,
            Visibility::Package => self.pkg == path.pkg,
            Visibility::Module => self.pkg == path.pkg && self.path == path.path,
        }
    }
}

impl fmt::Display for FQPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pkg.fmt(f)?;
        self.path.fmt(f)
    }
}

impl fmt::Debug for FQPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "FQPath", self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FQ<T> {
    pub path: FQPath,
    pub enclosing: Option<TSymbol>,
    pub symbol: T,
}

impl<T> FQ<T> {
    pub fn is_top_level(&self) -> bool {
        self.enclosing.is_none()
    }

    pub fn can_see<O>(&self, visibility: Visibility, symbol: &FQ<O>) -> bool {
        self.path.can_see(visibility, &symbol.path)
    }
}

impl<T: Display> fmt::Display for FQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.path.fmt(f)?;
        separator(f)?;
        if let Some(enclosing) = &self.enclosing {
            enclosing.fmt(f)?;
            separator(f)?;
        }
        self.symbol.fmt(f)
    }
}

impl<T: Display> fmt::Debug for FQ<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(f, "FQ", self)
    }
}

pub type FQSym = FQ<Symbol>;
pub type FQType = FQ<TSymbol>;

pub static NONE: TSymbol = TSymbol::known(KnownTSymbol::None);
pub static TRUE: TSymbol = TSymbol::known(KnownTSymbol::True);
pub static FALSE: TSymbol = TSymbol::known(KnownTSymbol::False);
pub static BOOLEAN: TSymbol = TSymbol::known(KnownTSymbol::Boolean);
pub static INTEGER: TSymbol = TSymbol::known(KnownTSymbol::Integer);

const fn std_type(symbol: KnownTSymbol) -> FQType {
    FQ {
        path: FQPath {
            pkg: Pkg::Std,
            path: Path::empty(),
        },
        enclosing: None,
        symbol: TSymbol::known(symbol),
    }
}

pub static FQ_NONE: FQType = std_type(KnownTSymbol::None);
pub static FQ_TRUE: FQType = std_type(KnownTSymbol::True);
pub static FQ_FALSE: FQType = std_type(KnownTSymbol::False);
pub static FQ_BOOLEAN: FQType = std_type(KnownTSymbol::Boolean);
pub static FQ_INTEGER: FQType = std_type(KnownTSymbol::Integer);
