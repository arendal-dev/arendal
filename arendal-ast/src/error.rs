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

    pub fn none() -> Self {
        Loc {
            _inner: Inner::None,
        }
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>, error: &dyn Error) -> fmt::Result {
        write!(f, "{:?}", error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Inner {
    None,
    Input(ArcStr, usize),
}

pub trait Error: fmt::Debug {}

#[derive(Debug)]
struct ErrorItem {
    _loc: Loc,
    _error: Box<dyn Error>,
}

type ErrorVec = Vec<ErrorItem>;

#[derive(Debug, Default)]
pub struct Errors {
    errors: ErrorVec,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for e in self.errors.iter() {
            e._loc.fmt(f, e._error.as_ref())?
        }
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, Errors>;

impl Errors {
    pub fn add<T: Error + 'static>(&mut self, loc: Loc, error: T) {
        let item = ErrorItem {
            _loc: loc,
            _error: Box::new(error),
        };
        self.errors.push(item);
    }

    pub fn append(&mut self, mut other: Errors) {
        self.errors.append(&mut other.errors)
    }

    pub fn append_result<T>(&mut self, result: Result<T>) {
        if let Err(other) = result {
            self.append(other);
        }
    }

    pub fn to_result<T>(self, value: T) -> Result<T> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }

    pub fn result_to_result<T>(mut self, result: Result<T>) -> Result<T> {
        match result {
            Ok(value) => self.to_result(value),
            Err(errors) => {
                self.append(errors);
                Err(self)
            }
        }
    }
}
