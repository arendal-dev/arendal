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
use ast::{Statement, problem::Result};
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

pub(crate) struct Env {
    symbols: Symbols,
    types: Types,
}

impl Env {
    fn empty() -> Self {
        Self {
            symbols: Symbols::default(),
            types: Types::default(),
        }
    }

    fn with_prelude() -> Self {
        Self::empty()
    }

    pub fn validate(statements: Vec<Statement>) -> Result<resolved::Resolved> {
        resolver::resolve(statements)
    }
}

pub struct Interactive {
    env: Env,
}

impl Interactive {
    pub fn new() -> Self {
        Self {
            env: Env::with_prelude(),
        }
    }

    pub fn eval(input: &str) -> Option<TypedValue> {
        panic!("TODO")
    }
}
