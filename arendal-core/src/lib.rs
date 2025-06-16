pub mod ast0;
pub mod context;
pub mod env0;
pub mod error;
pub mod id;
pub mod keyword;
pub mod symbol;
pub mod tst;
pub mod visibility;

pub use arcstr::{ArcStr, Substr, literal};
use ast::{AST, Statement, problem::Result};
pub use num::Integer;

use crate::{
    symbols::Symbols,
    types::{TypedValue, Types},
};

mod resolved;
mod resolver;
mod symbols;
mod typechecked;
mod typechecker;
mod types;

pub(crate) struct GlobalScope {
    symbols: Symbols,
    types: Types,
}

impl GlobalScope {
    fn empty() -> Self {
        Self {
            symbols: Symbols::default(),
            types: Types::default(),
        }
    }
}

pub(crate) struct Env {
    global: GlobalScope,
}

impl Env {
    fn new() -> Self {
        let global = GlobalScope::empty();
        // TODO: add prelude
        Env { global }
    }

    fn resolve(&self, ast: &AST) -> Result<resolved::Resolved> {
        resolver::resolve(&self.global, ast)
    }
}

pub struct Interactive {
    env: Env,
}

impl Interactive {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }

    pub fn eval(input: &str) -> Option<TypedValue> {
        panic!("TODO")
    }
}
