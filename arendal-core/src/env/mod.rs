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
    parent: Option<Arc<Scope<T>>>,
    values: HashMap<Symbol, T>,
}

impl<T: Clone> Scope<T> {
    fn new() -> Self {
        Scope {
            parent: None,
            values: Default::default(),
        }
    }

    fn create_child(&self) -> Self {
        Scope {
            parent: Some(Arc::new(self.clone())),
            values: Default::default(),
        }
    }

    fn get(&self, symbol: &Symbol) -> Option<T> {
        match self.values.get(symbol) {
            Some(v) => Some(v.clone()),
            None => self.parent.as_ref().and_then(|p| p.get(symbol)),
        }
    }

    pub(crate) fn set(&mut self, loc: &Loc, symbol: Symbol, value: T) -> Result<()> {
        if self.values.contains_key(&symbol) {
            loc.err(Error::DuplicateLocalSymbol(symbol))
        } else {
            self.values.insert(symbol, value);
            Ok(())
        }
    }
}
