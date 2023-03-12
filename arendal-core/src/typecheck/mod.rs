use crate::ast::Expression;
use crate::error::{Error, Result};
use crate::scope::Scope;
use crate::typed::TypedExpr;

mod expr;

pub fn expression(scope: &mut Scope, input: &Expression) -> Result<TypedExpr> {
    expr::check(scope, input)
}

#[derive(Debug)]
enum TypeError {
    InvalidType, // placeholder, temporary error
}

impl Error for TypeError {}
