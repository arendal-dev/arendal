mod expr;
mod value;

use ast::error::{Error, Errors, Result};
use ast::{BinaryOp, Loc, TExpr, Type, TypedExpr};

pub use value::Value;

#[derive(Debug)]
pub struct RuntimeError {}

impl Error for RuntimeError {}

pub type ValueResult = Result<Value>;

pub fn expression(expr: TypedExpr) -> ValueResult {
    expr::eval(expr)
}
