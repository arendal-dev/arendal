use std::collections::HashMap;

use ast::{
    position::Position,
    problem::{self, ErrorType, Problems, Result},
};
use num::Integer;

use crate::{symbol::FQSym, ttr::Expression, types::Type};

#[derive(Debug)]
pub struct SymbolDfn {
    position: Position,
    expr: Expression,
}

#[derive(Debug, Default)]
pub(crate) struct Symbols {
    values: HashMap<FQSym, SymbolDfn>,
}

impl Symbols {
    pub(crate) fn contains(&self, symbol: &FQSym) -> bool {
        self.values.contains_key(symbol)
    }

    pub(crate) fn get(&self, symbol: &FQSym) -> Option<&SymbolDfn> {
        self.values.get(symbol)
    }

    pub(crate) fn set(&mut self, symbol: FQSym, dfn: SymbolDfn) -> Result<()> {
        if self.values.contains_key(&symbol) {
            Error::DuplicateSymbol(symbol).at(dfn.position).to_err()
        } else {
            self.values.insert(symbol, dfn);
            Problems::ok(())
        }
    }
}

#[derive(Debug)]
enum Error {
    DuplicateSymbol(FQSym),
}

impl ErrorType for Error {
    fn at(self, position: Position) -> problem::Error {
        problem::Error::new(position, self)
    }
}
