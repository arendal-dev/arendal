mod tst;

use std::sync::Arc;

use im::HashMap;

use crate::{
    error::{Error, Loc, Result},
    symbol::Symbol,
    types::Types,
    values::{Value, Values},
};

#[derive(Debug, Clone, Default)]
pub struct Env {
    types: Types,
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
