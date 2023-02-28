use super::{ArcStr, Errors, Loc, Result};
use std::fmt;

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
}

impl core::ops::Deref for Identifier {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.name.deref()
    }
}

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
