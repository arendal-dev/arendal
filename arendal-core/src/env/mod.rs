mod prelude;
mod twi;
mod typecheck;

use crate::{
    ast,
    error::Result,
    symbol::{FQSym, Path, Pkg, Symbol, TSymbol},
    types::{Type, Types},
    value::{Value, Values},
};

#[derive(Debug, Clone, Default)]
struct Env {
    types: Types,
    values: Values,
}

#[derive(Debug)]
struct Package {
    id: Pkg,
    env: Env,
}

impl Default for Package {
    fn default() -> Self {
        Self {
            id: Pkg::Local,
            env: Default::default(),
        }
    }
}

pub struct Interactive {
    env: Env,
    path: Path,
}

impl Default for Interactive {
    fn default() -> Self {
        Interactive {
            env: Env::default(),
            path: Pkg::Local.empty(),
        }
    }
}

impl Interactive {
    pub fn module(&mut self, input: &ast::Module) -> Result<Value> {
        let module = typecheck::check(&self.env, &self.path, input)?;
        twi::interpret(&mut self.env, &module)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EnvError {
    DuplicateModule(Pkg, Path),
    DuplicateSymbol(FQSym),
    DuplicateType(Type),
    DuplicateVal(Symbol),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RuntimeError {
    UknownVal(Symbol),
    DivisionByZero,
    NotImplemented,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeCheckError {
    UnknownType(TSymbol),
    UnknownIdentifier(Symbol),
    InvalidType, // placeholder, temporary error
}
