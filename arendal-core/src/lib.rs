pub mod ast0;
pub mod context;
pub mod env;
pub mod error;
pub mod id;
pub mod keyword;
pub mod symbol;
pub mod tst;
pub mod visibility;

pub use arcstr::{ArcStr, Substr, literal};
use ast::{Statement, problem::Result};
pub use num::Integer;

mod itr;
mod typechecker;
mod types;
mod validator;

pub struct Env {}

impl Env {
    pub fn validate(statements: Vec<Statement>) -> Result<validator::ITR> {
        validator::validate(statements)
    }
}
