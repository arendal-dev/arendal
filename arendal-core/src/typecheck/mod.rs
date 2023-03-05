use crate::ast::Expression;
use crate::error::{Error, Result};
use crate::scope::Scope;
use crate::typed::TypedExpr;

mod expr;

pub struct Checked<T> {
    pub scope: Scope,
    pub it: T,
}

impl<T> Checked<T> {
    pub fn new(scope: Scope, it: T) -> Self {
        Checked { scope, it }
    }
}

pub type CheckedExpr = Checked<TypedExpr>;

pub fn expression(scope: Scope, input: Expression) -> Result<CheckedExpr> {
    expr::check(scope, input)
}

#[derive(Debug)]
enum TypeError {
    InvalidType, // placeholder, temporary error
}

impl Error for TypeError {}
