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

#[derive(Debug)]
pub enum EnvError {
    DuplicateModule(Pkg, Path),
    DuplicateSymbol(FQSym),
    DuplicateType(Type),
    DuplicateVal(Symbol),
}

impl Error for EnvError {}
