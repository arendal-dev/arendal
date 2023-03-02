pub mod value;

mod expr;

use core::error::{Error, Result};
use core::typed::TypedExpr;

#[derive(Debug)]
pub struct RuntimeError {}

impl Error for RuntimeError {}

pub type ValueResult = Result<value::Value>;

pub fn expression(expr: TypedExpr) -> ValueResult {
    expr::eval(expr)
}
