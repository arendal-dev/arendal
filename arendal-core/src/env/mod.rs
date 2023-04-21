mod prelude;
mod twi;
mod typecheck;

use crate::{
    ast,
    error::{Error, Result},
    symbol::{FQSym, Path, Pkg, Symbol},
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
    path: Path,
    interpreter: twi::Interpreter,
}

impl Default for Interactive {
    fn default() -> Self {
        let path = Pkg::Local.empty();
        Interactive {
            path: path.clone(),
            interpreter: Interpreter::new(Env::default(), path),
        }
    }
}

impl Interactive {
    pub fn module(&mut self, input: &ast::Module) -> Result<Value> {
        let typed = typecheck::TypeChecker::new(&self.interpreter.env, &self.path).module(input)?;
        self.interpreter.expression(&typed)
    }
}

#[derive(Debug)]
pub enum EnvError {
    DuplicateModule(Pkg, Path),
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
