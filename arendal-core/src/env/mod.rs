mod twi;

use im::HashMap;

use crate::{
    context::{Context, Type, Value},
    error::{Error, Loc, Result},
    symbol::FQSym,
    tst,
    visibility::{V, Visibility},
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
    pub(crate) types: Context,
    pub(crate) symbols: Symbols,
    values: Values,
}

impl Env {
    pub fn run(&mut self, input: &str) -> Result<Value> {
        let package = tst::check(self, input)?;
        twi::run(self, &package)
    }
}
