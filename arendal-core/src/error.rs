use crate::{
    ast::{parser::Enclosure, Q},
    keyword::Keyword,
    symbol::{FQPath, FQSym, FQType, Pkg, Symbol, TSymbol},
    types::Type,
    visibility::{Visibility, V},
};

use super::ArcStr;
use std::{collections::HashSet, fmt, hash::Hash, sync::Arc};

#[derive(Debug)]
pub struct Input {
    input: ArcStr,
    pos: usize,
}

#[derive(Debug, Clone)]
pub enum Loc {
    None,
    Input(Arc<Input>),
}

impl Loc {
    pub fn input(input: ArcStr, pos: usize) -> Self {
        Loc::Input(Arc::new(Input { input, pos }))
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>, error: &Error) -> fmt::Result {
        write!(f, "{:?}", error)
    }

    pub fn to_err<T>(self, error: Error) -> Result<T> {
        Err(ErrorVec::new(self.to_wrap(error)))
    }

    pub fn err<T>(&self, error: Error) -> Result<T> {
        self.clone().to_err(error)
    }

    pub fn to_wrap<T>(self, it: T) -> L<T> {
        L { loc: self, it }
    }

    pub fn wrap<T>(&self, it: T) -> L<T> {
        self.clone().to_wrap(it)
    }
}

impl PartialEq for Loc {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Loc {}

#[derive(Clone, PartialEq, Eq)]
pub struct L<T> {
    pub loc: Loc,
    pub it: T,
}

impl<T> L<T> {
    pub fn error(&self, error: Error) -> L<Error> {
        self.loc.wrap(error)
    }

    pub fn err<R>(&self, error: Error) -> Result<R> {
        self.loc.err(error)
    }

    pub fn map<U, F>(self, f: F) -> L<U>
    where
        F: FnOnce(T) -> U,
    {
        self.loc.to_wrap(f(self.it))
    }

    pub fn to_lv(self, visibility: Visibility) -> L<V<T>> {
        self.loc.to_wrap(visibility.wrap(self.it))
    }
}

impl<T: fmt::Debug> fmt::Debug for L<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.it.fmt(f)
    }
}

#[derive(Debug)]
pub struct ErrorVec {
    errors: Vec<L<Error>>,
}

impl fmt::Display for ErrorVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for e in self.errors.iter() {
            e.loc.fmt(f, &e.it)?
        }
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, ErrorVec>;

impl ErrorVec {
    fn new(item: L<Error>) -> Self {
        Self { errors: vec![item] }
    }

    fn add(&mut self, item: L<Error>) {
        self.errors.push(item);
    }

    fn append(&mut self, mut other: ErrorVec) {
        self.errors.append(&mut other.errors);
    }

    pub fn contains(&self, error: &Error) -> bool {
        self.errors.iter().map(|i| &i.it).any(|e| e == error)
    }

    fn for_each_error<F>(&self, mut f: F)
    where
        F: FnMut(&Error),
    {
        self.errors.iter().for_each(|e| f(&e.it))
    }

    fn collect_vec<T, F>(&self, f: F) -> Vec<T>
    where
        F: Fn(&Error) -> Option<T>,
    {
        let mut result = Vec::default();
        self.for_each_error(|e| {
            if let Some(t) = f(e) {
                result.push(t)
            }
        });
        result
    }

    fn collect_set<T: Eq + Hash, F>(&self, f: F) -> HashSet<T>
    where
        F: Fn(&Error) -> Option<T>,
    {
        let mut result = HashSet::default();
        self.for_each_error(|e| {
            if let Some(t) = f(e) {
                result.insert(t);
            }
        });
        result
    }

    pub(crate) fn missing_symbol_deps(&self) -> HashSet<FQSym> {
        self.collect_set(|e| {
            if let Error::MissingSymbolDependency(s) = e {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    pub(crate) fn missing_local_symbol_deps(&self) -> HashSet<Symbol> {
        self.collect_set(|e| {
            if let Error::MissingLocalSymbolDependency(s) = e {
                Some(s.clone())
            } else {
                None
            }
        })
    }
}

// Error accumulator and builder.
#[derive(Debug, Default)]
pub struct Errors {
    errors: Option<ErrorVec>,
}

impl Errors {
    pub fn add(&mut self, error: L<Error>) {
        match &mut self.errors {
            Some(e) => e.add(error),
            None => self.errors = Some(ErrorVec::new(error)),
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
        F: FnOnce() -> T,
    {
        match self.errors {
            None => Ok(supplier()),
            Some(e) => Err(e),
        }
    }

    pub fn to_merged_result<T>(self, result: Result<T>) -> Result<T> {
        match self.errors {
            None => result,
            Some(mut e) => match result {
                Ok(_) => Err(e),
                Err(e2) => {
                    e.append(e2);
                    Err(e)
                }
            },
        }
    }

    pub fn to_err<T>(mut self, error: L<Error>) -> Result<T> {
        self.add(error);
        Err(self.errors.unwrap())
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
    EmptyPackage,
    DuplicateModule(FQPath),
    UnexpectedPackage(Pkg, Pkg),
    ExpressionExpected,
    ExpressionNotExpected,
    LValueExpected,
    AssignmentExpected,
    EndOfItemExpected,
    KeywordExpected(Keyword),
    CloseExpected(Enclosure),
    TSymbolAfterTypeExpected,
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
    DuplicateLocalSymbol(Symbol),
    UnableToResolveSymbol(Q<Symbol>),
    UnableToResolveType(Q<TSymbol>),
    TLExpressionInNonRootModule,
    SymbolNotVisible(FQSym),
    TypeNotVisible(FQType),
    OnlyOneExpressionAllowed,
    MissingSymbolDependency(FQSym),       // internal
    MissingLocalSymbolDependency(Symbol), // internal
    // Type checking and runtime
    TypeMismatch(Arc<TypeMismatch>),
    IntegerExpected(Type),
    SingletonExpected(Type),
    InvalidType, // placeholder, temporary error
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

    pub fn merge3<T1, T2, T3>(
        r1: Result<T1>,
        r2: Result<T2>,
        r3: Result<T3>,
    ) -> Result<(T1, T2, T3)> {
        Self::merge(Self::merge(r1, r2), r3).map(|((t1, t2), t3)| (t1, t2, t3))
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
