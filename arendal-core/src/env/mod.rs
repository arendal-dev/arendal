mod tst;

use im::HashMap;

use crate::{
    error::{Error, Loc, Result},
    symbol::{FQSym, Symbol},
    types::{Type, Types},
    values::Value,
    visibility::{Visibility, V},
};

#[derive(Debug, Clone)]
struct SymbolMap<T: Clone> {
    values: HashMap<FQSym, V<T>>,
}

impl<T: Clone> Default for SymbolMap<T> {
    fn default() -> Self {
        Self {
            values: Default::default(),
        }
    }
}

impl<T: Clone> SymbolMap<T> {
    pub(crate) fn get(&self, symbol: &FQSym) -> Option<V<T>> {
        self.values.get(symbol).cloned()
    }

    pub(crate) fn set(
        &mut self,
        loc: &Loc,
        symbol: FQSym,
        visibility: Visibility,
        value: T,
    ) -> Result<()> {
        if self.values.contains_key(&symbol) {
            loc.err(Error::DuplicateSymbol(symbol))
        } else {
            self.values.insert(symbol, visibility.wrap(value));
            Ok(())
        }
    }
}

type Symbols = SymbolMap<Type>;
type Values = SymbolMap<Value>;

#[derive(Debug, Clone, Default)]
pub struct Env {
    types: Types,
    symbols: Symbols,
    values: Values,
}

impl Env {
    pub fn run(&mut self, input: &str) -> Result<Value> {
        tst::run(self, input)
    }
}

#[derive(Debug, Clone)]
struct Scope<T: Clone> {
    all: HashMap<Symbol, T>,
    current: HashMap<Symbol, T>,
}

impl<T: Clone> Default for Scope<T> {
    fn default() -> Self {
        Scope {
            all: Default::default(),
            current: Default::default(),
        }
    }
}

impl<T: Clone> Scope<T> {
    fn create_child(&self) -> Self {
        Scope {
            all: self.all.clone(),
            current: Default::default(),
        }
    }

    fn contains(&self, symbol: &Symbol) -> bool {
        self.all.contains_key(symbol)
    }

    fn get(&self, symbol: &Symbol) -> Option<T> {
        self.current
            .get(symbol)
            .or_else(|| self.all.get(symbol))
            .cloned()
    }

    fn set(&mut self, loc: &Loc, symbol: Symbol, value: T) -> Result<()> {
        if self.current.contains_key(&symbol) {
            loc.err(Error::DuplicateLocalSymbol(symbol))
        } else {
            self.current.insert(symbol.clone(), value.clone());
            self.all.insert(symbol, value);
            Ok(())
        }
    }
}
