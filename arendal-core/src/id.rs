use crate::keyword::Keyword;
use crate::ArcStr;

use std::cmp;
use std::fmt;
use std::rc::Rc;

trait Identifier:
    Clone + fmt::Debug + fmt::Display + cmp::PartialEq + cmp::Eq + std::hash::Hash
{
    fn as_str(&self) -> &str;
}

// Package id will eventually be some kind of hash, but we start with the same restrictions as an id for now.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    name: ArcStr,
}

impl PackageId {
    pub fn new(name: ArcStr) -> Result<Id, IdError> {
        if name.is_empty() {
            return Err(IdError::Empty);
        }
        if let Some(k) = Keyword::parse(name.as_str()) {
            return Err(IdError::Keyword(k));
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_lowercase() {
                    return Err(IdError::InvalidInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return Err(IdError::InvalidChar(i, c));
                }
            }
        }
        Ok(Id { name: name.into() })
    }
}

impl Identifier for PackageId {
    fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PackageId({})", self.as_str())
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl core::ops::Deref for PackageId {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.name.deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Id {
    name: ArcStr,
}

impl Id {
    pub fn new(name: ArcStr) -> Result<Id, IdError> {
        if name.is_empty() {
            return Err(IdError::Empty);
        }
        if let Some(k) = Keyword::parse(name.as_str()) {
            return Err(IdError::Keyword(k));
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_lowercase() {
                    return Err(IdError::InvalidInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return Err(IdError::InvalidChar(i, c));
                }
            }
        }
        Ok(Id { name: name.into() })
    }
}

impl Identifier for Id {
    fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({})", self.as_str())
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl core::ops::Deref for Id {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.name.deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TypeId {
    name: ArcStr,
}

impl TypeId {
    pub fn new(name: ArcStr) -> Result<TypeId, IdError> {
        if name.is_empty() {
            return Err(IdError::TypeEmpty);
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_uppercase() {
                    return Err(IdError::InvalidTypeInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return Err(IdError::InvalidChar(i, c));
                }
            }
        }
        Ok(TypeId { name: name.into() })
    }
}

impl Identifier for TypeId {
    fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Debug for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeId({})", self.as_str())
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl core::ops::Deref for TypeId {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.name.deref()
    }
}

#[derive(Debug)]
pub enum IdError {
    Empty,
    TypeEmpty,
    Keyword(Keyword),
    InvalidInitial(char),
    InvalidTypeInitial(char),
    InvalidChar(usize, char),
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum InnerPath {
    Std,
    StdChild(Id),
    ThisPackage,
    ThisPackageChild(Id),
    Package(PackageId),
    PackageChild(PackageId, Id),
    Module(Rc<InnerPath>, Id),
}

impl fmt::Display for InnerPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Std => f.write_str("std"),
            Self::StdChild(id) => write!(f, "std::{}", id),
            Self::ThisPackage => f.write_str("package"),
            Self::ThisPackageChild(id) => write!(f, "package::{}", id),
            Self::Package(id) => id.fmt(f),
            Self::PackageChild(idp, idm) => write!(f, "{}::{}", idp, idm),
            Self::Module(parent, id) => write!(f, "{}::{}", *parent, id),
        }
    }
}

impl fmt::Debug for InnerPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Path {
    inner: InnerPath,
}

impl Path {
    #[inline]
    fn new(inner: InnerPath) -> Self {
        Path { inner }
    }

    pub fn std() -> Self {
        Self::new(InnerPath::Std)
    }

    pub fn this_package() -> Self {
        Self::new(InnerPath::ThisPackage)
    }

    pub fn package(id: PackageId) -> Self {
        Self::new(InnerPath::Package(id))
    }

    pub fn child(&self, id: Id) -> Self {
        match &self.inner {
            InnerPath::Std => Self::new(InnerPath::StdChild(id)),
            InnerPath::ThisPackage => Self::new(InnerPath::ThisPackageChild(id)),
            InnerPath::Package(idp) => Self::new(InnerPath::PackageChild(idp.clone(), id)),
            _ => Self::new(InnerPath::Module(Rc::new(self.inner.clone()), id)),
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl crate::error::Error for IdError {}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FQId {
    path: Path,
    id: Id,
}

impl fmt::Display for FQId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.path, self.id)
    }
}

impl fmt::Debug for FQId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FQTypeId {
    path: Path,
    id: TypeId,
}

impl FQTypeId {
    pub fn new(path: Path, id: TypeId) -> Self {
        FQTypeId { path, id }
    }

    pub fn std(id: TypeId) -> Self {
        Self::new(Path::std(), id)
    }
}

impl fmt::Display for FQTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.path, self.id)
    }
}

impl fmt::Debug for FQTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}
