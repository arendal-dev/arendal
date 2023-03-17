//use crate::id::Id;
use crate::keyword::Keyword;
use crate::ArcStr;

use std::cmp;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PackageId {
    Std,
    Current,
    Package(Identifier),
}

impl PackageId {
    pub fn new(name: ArcStr) -> Result<Identifier, IdError> {
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
        Ok(Identifier { name: name.into() })
    }
}

impl fmt::Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PackageId({})", "self.as_str()")
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TODO")
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    name: ArcStr,
}

impl Identifier {
    pub fn new(name: ArcStr) -> Result<Identifier, IdError> {
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
        Ok(Identifier { name: name.into() })
    }

    fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({})", self.as_str())
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl core::ops::Deref for Identifier {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.name.deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TypeIdentifier {
    name: ArcStr,
}

impl TypeIdentifier {
    pub fn new(name: ArcStr) -> Result<TypeIdentifier, IdError> {
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
        Ok(TypeIdentifier { name: name.into() })
    }

    fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Debug for TypeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeId({})", self.as_str())
    }
}

impl fmt::Display for TypeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl core::ops::Deref for TypeIdentifier {
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
    StdChild(Identifier),
    ThisPackage,
    ThisPackageChild(Identifier),
    Package(PackageId),
    PackageChild(PackageId, Identifier),
    Module(Rc<InnerPath>, Identifier),
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

    pub fn child(&self, id: Identifier) -> Self {
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
    id: Identifier,
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
    id: TypeIdentifier,
}

impl FQTypeId {
    pub fn new(path: Path, id: TypeIdentifier) -> Self {
        FQTypeId { path, id }
    }

    pub fn std(id: TypeIdentifier) -> Self {
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
