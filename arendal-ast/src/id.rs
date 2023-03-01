use super::{ArcStr, Errors, Loc, Result};
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Identifier {
    name: ArcStr,
}

impl Identifier {
    pub fn on(loc: Loc, name: &str) -> Result<Identifier> {
        if name.is_empty() {
            return err(loc, Error::Empty);
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_lowercase() {
                    return err(loc, Error::InvalidInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return err(loc, Error::InvalidChar(i, c));
                }
            }
        }
        Ok(Identifier { name: name.into() })
    }

    pub fn new(name: &str) -> Result<Identifier> {
        Self::on(Loc::none(), name)
    }

    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identifier({})", self.as_str())
    }
}

impl core::ops::Deref for Identifier {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.name.deref()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TypeIdentifier {
    name: ArcStr,
}

impl TypeIdentifier {
    pub fn on(loc: Loc, name: &str) -> Result<TypeIdentifier> {
        if name.is_empty() {
            return err(loc, Error::TypeEmpty);
        }
        for (i, c) in name.char_indices() {
            if i == 0 {
                if !c.is_ascii_alphabetic() || !c.is_ascii_uppercase() {
                    return err(loc, Error::InvalidTypeInitial(c));
                }
            } else {
                if !c.is_ascii_alphanumeric() {
                    return err(loc, Error::InvalidChar(i, c));
                }
            }
        }
        Ok(TypeIdentifier { name: name.into() })
    }

    pub fn new(name: &str) -> Result<TypeIdentifier> {
        Self::on(Loc::none(), name)
    }

    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Display for TypeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Debug for TypeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeIdentifier({})", self.as_str())
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
enum Error {
    Empty,
    TypeEmpty,
    InvalidInitial(char),
    InvalidTypeInitial(char),
    InvalidChar(usize, char),
}

impl super::Error for Error {}

#[inline]
fn err<T>(loc: Loc, error: Error) -> Result<T> {
    Err(Errors::new(loc, error))
}
