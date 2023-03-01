mod expr;
mod value;

use ast::{BinaryOp, Error, Errors, Loc, Result, TExpr, Type, TypedExpr};

pub use value::Value;

#[derive(Debug)]
pub struct RuntimeError {}

impl Error for RuntimeError {}

pub type ValueResult = Result<Value>;

pub fn expression(expr: TypedExpr) -> ValueResult {
    expr::eval(expr)
}
