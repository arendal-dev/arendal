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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Id {
    name: ArcStr,
}

impl Id {
    pub fn new(name: &str) -> Result<Id, IdError> {
        if name.is_empty() {
            return Err(IdError::Empty);
        }
        if let Some(k) = Keyword::parse(name) {
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
    pub fn new(name: &str) -> Result<TypeId, IdError> {
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
enum PathParent {
    Std,
    ThisModule,
    Other(Path),
}

impl fmt::Display for PathParent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Std => f.write_str("std::"),
            Self::ThisModule => f.write_str("self::"),
            Self::Other(p) => p.fmt(f),
        }
    }
}

impl fmt::Debug for PathParent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct InnerPath {
    parent: PathParent,
    segment: Id,
}

impl fmt::Display for InnerPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.parent.fmt(f);
        self.segment.fmt(f)
    }
}

impl fmt::Debug for InnerPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Path {
    inner: Rc<InnerPath>,
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
        self.path.fmt(f);
        self.id.fmt(f)
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

impl fmt::Display for FQTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.path.fmt(f);
        self.id.fmt(f)
    }
}

impl fmt::Debug for FQTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}
