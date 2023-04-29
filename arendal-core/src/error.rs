use super::ArcStr;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Loc {
    _inner: Inner,
}

impl Loc {
    pub fn input(input: ArcStr, pos: usize) -> Self {
        Loc {
            _inner: Inner::Input(input, pos),
        }
    }

    pub const fn none() -> Self {
        Loc {
            _inner: Inner::None,
        }
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>, error: &ErrorKind) -> fmt::Result {
        write!(f, "{:?}", error)
    }
}

impl PartialEq for Loc {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Loc {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Inner {
    None,
    Input(ArcStr, usize),
}

#[derive(Debug)]
struct ErrorItem {
    loc: Loc,
    error: ErrorKind,
}

type ErrorVec = Vec<ErrorItem>;

#[derive(Debug)]
pub struct Error {
    errors: ErrorVec,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for e in self.errors.iter() {
            e.loc.fmt(f, &e.error)?
        }
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    fn new(loc: Loc, error: ErrorKind) -> Self {
        Self {
            errors: vec![ErrorItem { loc, error }],
        }
    }

    #[inline]
    pub fn err<E: Into<ErrorKind>, T>(loc: Loc, error: E) -> Result<T> {
        Err(Self::new(loc, error.into()))
    }

    pub fn merge<T1, T2, TO, O>(r1: Result<T1>, r2: Result<T2>, op: O) -> Result<TO>
    where
        O: FnOnce(T1, T2) -> Result<TO>,
    {
        match (r1, r2) {
            (Err(mut e1), Err(e2)) => {
                e1.append(e2);
                Err(e1)
            }
            (Err(e1), Ok(_)) => Err(e1),
            (Ok(_), Err(e2)) => Err(e2),
            (Ok(t1), Ok(t2)) => op(t1, t2),
        }
    }

    fn add(&mut self, loc: Loc, error: ErrorKind) {
        self.errors.push(ErrorItem { loc, error });
    }

    fn append(&mut self, mut other: Error) {
        self.errors.append(&mut other.errors);
    }
}

// Error accumulator and builder.
#[derive(Debug, Default)]
pub struct Errors {
    errors: Option<Error>,
}

impl Errors {
    pub fn add<E: Into<ErrorKind>>(&mut self, loc: Loc, error: E) {
        match &mut self.errors {
            Some(e) => e.add(loc, error.into()),
            None => self.errors = Some(Error::new(loc, error.into())),
        }
    }

    pub fn add_result<T>(&mut self, result: Result<T>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(others) => {
                match &mut self.errors {
                    Some(e) => e.append(others),
                    None => self.errors = Some(others),
                };
                None
            }
        }
    }

    pub fn to_result<T>(self, value: T) -> Result<T> {
        match self.errors {
            None => Ok(value),
            Some(e) => Err(e),
        }
    }

    pub fn to_unit_result(self) -> Result<()> {
        self.to_result(())
    }

    pub fn to_lazy_result<T, F>(self, supplier: F) -> Result<T>
    where
        F: FnOnce(()) -> T,
    {
        match self.errors {
            None => Ok(supplier(())),
            Some(e) => Err(e),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Symbol(crate::symbol::SymbolError),
    Parser(crate::parser::ParserError),
    Types(crate::types::TypesError),
    Value(crate::value::ValueError),
    Env(crate::env::EnvError),
    Runtime(crate::env::RuntimeError),
    TypeCheck(crate::env::TypeCheckError),
}

impl From<crate::symbol::SymbolError> for ErrorKind {
    fn from(value: crate::symbol::SymbolError) -> Self {
        ErrorKind::Symbol(value)
    }
}

impl From<crate::parser::ParserError> for ErrorKind {
    fn from(value: crate::parser::ParserError) -> Self {
        ErrorKind::Parser(value)
    }
}

impl From<crate::types::TypesError> for ErrorKind {
    fn from(value: crate::types::TypesError) -> Self {
        ErrorKind::Types(value)
    }
}

impl From<crate::value::ValueError> for ErrorKind {
    fn from(value: crate::value::ValueError) -> Self {
        ErrorKind::Value(value)
    }
}

impl From<crate::env::EnvError> for ErrorKind {
    fn from(value: crate::env::EnvError) -> Self {
        ErrorKind::Env(value)
    }
}

impl From<crate::env::RuntimeError> for ErrorKind {
    fn from(value: crate::env::RuntimeError) -> Self {
        ErrorKind::Runtime(value)
    }
}

impl From<crate::env::TypeCheckError> for ErrorKind {
    fn from(value: crate::env::TypeCheckError) -> Self {
        ErrorKind::TypeCheck(value)
    }
}
