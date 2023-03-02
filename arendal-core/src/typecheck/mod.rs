use crate::ast::Expression;
use crate::error::{Error, Result};
use crate::typed::TypedExpr;

mod expr;

pub fn expression(input: Expression) -> Result<TypedExpr> {
    expr::check(input)
}

#[derive(Debug)]
enum TypeError {
    InvalidType, // placeholder, temporary error
}

impl Error for TypeError {}
