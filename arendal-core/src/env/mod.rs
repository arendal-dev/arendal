mod twi;

use im::HashMap;

use crate::{
    error::{Error, Loc, Result},
    symbol::{FQSym, Symbol},
    tst,
    types::{Type, Types},
    values::Value,
    visibility::{Visibility, V},
};

#[derive(Debug, Default, Clone)]
pub(crate) struct Symbols {
    values: HashMap<FQSym, V<Type>>,
}

impl Symbols {
    pub(crate) fn contains(&self, symbol: &FQSym) -> bool {
        self.values.contains_key(symbol)
    }

    pub(crate) fn get(&self, symbol: &FQSym) -> Option<V<Type>> {
        self.values.get(symbol).cloned()
    }

    pub(crate) fn set(
        &mut self,
        loc: &Loc,
        symbol: FQSym,
        visibility: Visibility,
        tipo: Type,
    ) -> Result<()> {
        if self.values.contains_key(&symbol) {
            loc.err(Error::DuplicateSymbol(symbol))
        } else {
            self.values.insert(symbol, visibility.wrap(tipo));
            Ok(())
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Values {
    values: HashMap<FQSym, Value>,
}

impl Values {
    pub(crate) fn contains(&self, symbol: &FQSym) -> bool {
        self.values.contains_key(symbol)
    }

    pub(crate) fn get(&self, symbol: &FQSym) -> Option<Value> {
        self.values.get(symbol).cloned()
    }

    pub(crate) fn set(&mut self, loc: &Loc, symbol: FQSym, value: Value) -> Result<()> {
        if self.values.contains_key(&symbol) {
            loc.err(Error::DuplicateSymbol(symbol))
        } else {
            self.values.insert(symbol, value);
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Env {
    pub(crate) types: Types,
    pub(crate) symbols: Symbols,
    values: Values,
}

impl Env {
    pub fn run(&mut self, input: &str) -> Result<Value> {
        let package = tst::check(self, input)?;
        twi::interpret(self, &package)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Scope<T: Clone> {
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
    pub(crate) fn create_child(&self) -> Self {
        Scope {
            all: self.all.clone(),
            current: Default::default(),
        }
    }

    pub(crate) fn contains(&self, symbol: &Symbol) -> bool {
        self.all.contains_key(symbol)
    }

    pub(crate) fn get(&self, symbol: &Symbol) -> Option<T> {
        self.current
            .get(symbol)
            .or_else(|| self.all.get(symbol))
            .cloned()
    }

    pub(crate) fn set(&mut self, loc: &Loc, symbol: Symbol, value: T) -> Result<()> {
        if self.current.contains_key(&symbol) {
            loc.err(Error::DuplicateLocalSymbol(symbol))
        } else {
            self.current.insert(symbol.clone(), value.clone());
            self.all.insert(symbol, value);
            Ok(())
        }
    }
}
