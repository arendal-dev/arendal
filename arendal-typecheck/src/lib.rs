mod expr;

use ast::{Error, Expression, Result, TypedExpr};

// 'static here means that L is owned
pub fn expression(input: Expression) -> Result<TypedExpr> {
    expr::check(input)
}

#[derive(Debug)]
enum TypeError {
    InvalidType, // placeholder, temporary error
}

impl Error for TypeError {}
