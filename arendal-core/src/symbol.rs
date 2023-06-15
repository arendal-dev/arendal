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
pub struct Path {
    path: Arc<Vec<Symbol>>,
}

impl Path {
    pub fn new(path: Vec<Symbol>) -> Self {
        Self {
            path: Arc::new(path),
        }
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

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
pub struct FQPath {
    pub(crate) pkg: Pkg,
    pub(crate) path: Path,
}

impl FQPath {
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

    fn can_see(&self, visibility: Visibility, path: &FQPath) -> bool {
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
struct TLData<T> {
    path: FQPath,
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

    pub fn path(&self) -> FQPath {
        match self {
            Self::TopLevel(t) => t.data.path.clone(),
            Self::Member(m) => m.data.top_level.path(),
        }
    }

    pub fn can_see(&self, visibility: Visibility, symbol: &FQSym) -> bool {
        self.path().can_see(visibility, &symbol.path())
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
    fn get_known(path: &FQPath, symbol: &TSymbol) -> Option<Self> {
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

    fn top_level(path: FQPath, symbol: TSymbol) -> Self {
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

    pub(crate) fn member_sym(&self, loc: &Loc, symbol: Symbol) -> Result<FQSym> {
        if self.is_top_level() {
            Ok(FQSym::Member(Member {
                data: Arc::new(MemberData {
                    top_level: self.clone(),
                    symbol,
                }),
            }))
        } else {
            loc.err(Error::TopLevelTypeExpected(self.clone()))
        }
    }

    pub(crate) fn member_type(&self, loc: &Loc, symbol: TSymbol) -> Result<Self> {
        if self.is_top_level() {
            Ok(Self::Member(Member {
                data: Arc::new(MemberData {
                    top_level: self.clone(),
                    symbol,
                }),
            }))
        } else {
            loc.err(Error::TopLevelTypeExpected(self.clone()))
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

    pub fn path(&self) -> FQPath {
        match self {
            Self::TopLevel(t) => t.data.path.clone(),
            Self::Member(m) => m.data.top_level.path(),
            _ => Pkg::Std.empty(),
        }
    }

    pub fn can_see(&self, visibility: Visibility, tipo: &FQType) -> bool {
        self.path().can_see(visibility, &tipo.path())
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
