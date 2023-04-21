mod prelude;
mod twi;
mod typecheck;

use crate::{
    ast::Expression,
    error::{Error, Result},
    symbol::{FQSym, ModulePath, Pkg, Symbol},
    types::{Type, Types},
    value::{Value, Values},
};

use self::twi::Interpreter;

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
    pkg: Pkg,
    path: ModulePath,
    interpreter: twi::Interpreter,
}

impl Default for Interactive {
    fn default() -> Self {
        Interactive {
            pkg: Pkg::Local,
            path: ModulePath::empty(),
            interpreter: Interpreter::new(Env::default(), Pkg::Local, ModulePath::empty()),
        }
    }
}

impl Interactive {
    pub fn expression(&mut self, input: &Expression) -> Result<Value> {
        let typed =
            typecheck::TypeChecker::new(&self.interpreter.env, self.pkg.clone(), self.path.clone())
                .expression(input)?;
        self.interpreter.expression(&typed)
    }
}

#[derive(Debug)]
pub enum EnvError {
    DuplicateModule(Pkg, ModulePath),
    DuplicateSymbol(FQSym),
    DuplicateType(Type),
    DuplicateVal(Symbol),
}

impl Error for EnvError {}

#[derive(Debug)]
enum TypeCheckError {
    UnknownIdentifier(Symbol),
    InvalidType, // placeholder, temporary error
}

impl Error for TypeCheckError {}
