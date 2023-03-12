use super::ArcStr;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug)]
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
    pub fn new<T: Error + 'static>(loc: Loc, error: T) -> Self {
        let mut errors: Errors = Errors {
            errors: Default::default(),
        };
        errors.add(loc, error);
        errors
    }

    #[inline]
    pub fn err<T, E: Error + 'static>(loc: Loc, error: E) -> Result<T> {
        Err(Self::new(loc, error))
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

    pub fn add_to<T, E: Error + 'static>(result: Result<T>, loc: Loc, error: E) -> Result<T> {
        match result {
            Ok(_) => Err(Self::new(loc, error)),
            Err(mut e) => {
                e.add(loc, error);
                Err(e)
            }
        }
    }

    pub fn add<T: Error + 'static>(&mut self, loc: Loc, error: T) {
        let item = ErrorItem {
            _loc: loc,
            _error: Box::new(error),
        };
        self.errors.push(item);
    }

    fn append(&mut self, mut other: Errors) {
        self.errors.append(&mut other.errors);
    }
}

#[derive(Debug, Default)]
pub struct ErrorAcc {
    errors: Option<Errors>,
}

impl ErrorAcc {
    pub fn add<T: Error + 'static>(&mut self, loc: Loc, error: T) {
        match &mut self.errors {
            Some(e) => e.add(loc, error),
            None => self.errors = Some(Errors::new(loc, error)),
        }
    }

    // Ok value is lost
    pub fn add_result<T>(&mut self, mut result: Result<T>) {
        if let Err(others) = result {
            match &mut self.errors {
                Some(e) => e.append(others),
                None => self.errors = Some(others),
            }
        }
    }

    pub fn to_result<T>(self, value: T) -> Result<T> {
        match self.errors {
            None => Ok(value),
            Some(e) => Err(e),
        }
    }

    pub fn to_err<T>(self) -> Result<T> {
        Err(self.errors.unwrap())
    }
}
