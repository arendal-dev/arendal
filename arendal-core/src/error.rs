use crate::{
    keyword::Keyword,
    parser::Enclosure,
    symbol::{FQSym, FQType, Path, Pkg, Symbol, TSymbol},
    types::Type,
};

use super::ArcStr;
use std::{fmt, sync::Arc};

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

    fn fmt(&self, f: &mut fmt::Formatter<'_>, error: &Error) -> fmt::Result {
        write!(f, "{:?}", error)
    }

    pub fn err<T>(&self, error: Error) -> Result<T> {
        Err(ErrorVec::new(self.clone(), error.into()))
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
    error: Error,
}

#[derive(Debug)]
pub struct ErrorVec {
    errors: Vec<ErrorItem>,
}

impl fmt::Display for ErrorVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for e in self.errors.iter() {
            e.loc.fmt(f, &e.error)?
        }
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, ErrorVec>;

impl ErrorVec {
    fn new(loc: Loc, error: Error) -> Self {
        Self {
            errors: vec![ErrorItem { loc, error }],
        }
    }

    fn add(&mut self, loc: Loc, error: Error) {
        self.errors.push(ErrorItem { loc, error });
    }

    fn append(&mut self, mut other: ErrorVec) {
        self.errors.append(&mut other.errors);
    }

    pub fn contains(&self, error: &Error) -> bool {
        self.errors.iter().map(|i| &i.error).any(|e| e == error)
    }
}

// Error accumulator and builder.
#[derive(Debug, Default)]
pub struct Errors {
    errors: Option<ErrorVec>,
}

impl Errors {
    pub fn add(&mut self, loc: Loc, error: Error) {
        match &mut self.errors {
            Some(e) => e.add(loc, error.into()),
            None => self.errors = Some(ErrorVec::new(loc, error.into())),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    // Tokenizer
    UnexpectedChar(char),
    // Lexer
    InvalidClose(Enclosure),
    UnexpectedToken,
    // Parser
    ExpressionExpected,
    LValueExpected,
    AssignmentExpected,
    EndOfItemExpected,
    ParsingError, // placeholder, temporary error
    // Symbols & type checking
    SymbolEmpty,
    TSymbolEmpty,
    SymbolKeywordFound(Keyword),
    SymbolInvalidInitial(char),
    TSymbolInvalidInitial(char),
    SymbolInvalidChar(usize, char),
    SymbolExpected(TSymbol),
    TSymbolExpected(Symbol),
    TopLevelTypeExpected(FQType),
    UnknownType(FQType),
    DuplicateType(FQType),
    UnknownSymbol(FQSym),
    DuplicateSymbol(FQSym),
    UnknownLocalType(TSymbol),
    DuplicateLocalType(TSymbol),
    UnknownLocalSymbol(Symbol),
    DuplicateLocalSymbol(TSymbol),
    // Type checking and runtime
    TypeMismatch(Arc<TypeMismatch>),
    SingletonExpected(Type),
    InvalidType, // placeholder, temporary error
    DuplicateModule(Pkg, Path),
    DivisionByZero,
    NotImplemented,
}

impl Error {
    pub fn merge<T1, T2>(r1: Result<T1>, r2: Result<T2>) -> Result<(T1, T2)> {
        match (r1, r2) {
            (Err(mut e1), Err(e2)) => {
                e1.append(e2);
                Err(e1)
            }
            (Err(e1), Ok(_)) => Err(e1),
            (Ok(_), Err(e2)) => Err(e2),
            (Ok(t1), Ok(t2)) => Ok((t1, t2)),
        }
    }

    pub fn type_mismatch(expected: Type, actual: Type) -> Self {
        Self::TypeMismatch(Arc::new(TypeMismatch { expected, actual }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMismatch {
    expected: Type,
    actual: Type,
}
