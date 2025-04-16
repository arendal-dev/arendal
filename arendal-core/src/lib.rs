pub mod ast0;
pub mod context;
pub mod env;
pub mod error;
pub mod id;
pub mod keyword;
pub mod symbol;
pub mod tst;
pub mod visibility;

use ::ast::{AST, problem::Result, stmt::Statement};
pub use arcstr::{ArcStr, Substr, literal};
pub use num::Integer;

mod validator;

pub struct Env {}

impl Env {
    pub fn validate(statements: Vec<Statement>) -> Result<AST<validator::Valid>> {
        validator::validate(statements)
    }
}
