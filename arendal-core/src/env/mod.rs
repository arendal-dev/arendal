mod prelude;
mod typed;

use crate::{
    error::Result,
    symbol::{Path, Pkg},
    types::Types,
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
    pub fn run(&mut self, input: &str) -> Result<Value> {
        typed::run(&mut self.env, &self.path, input)
    }
}
