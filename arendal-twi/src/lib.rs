mod expr;
mod value;

use ast::error::{Error, Result};
use ast::{Loc, SafeLoc, TypedExpression};

pub use value::Value;

#[derive(Debug)]
pub struct RuntimeError {
    loc: Box<dyn SafeLoc>,
}

impl RuntimeError {
    fn new<L: Loc + 'static>(loc: L) -> Self {
        RuntimeError { loc: Box::new(loc) }
    }
}

impl Error for RuntimeError {}

pub type ValueResult = Result<Value>;

pub fn expression<L: Loc + 'static>(expr: TypedExpression<L>) -> ValueResult {
    expr::eval(expr)
}
