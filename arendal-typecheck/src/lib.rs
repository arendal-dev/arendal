mod expr;

use ast::error::{Error, Result};
use ast::{Expression, Loc, TExpr, TypedExpr};

// 'static here means that L is owned
pub fn expression(input: Expression) -> Result<TypedExpr> {
    expr::check(input)
}

#[derive(Debug)]
enum TypeError {
    InvalidType, // placeholder, temporary error
}

impl Error for TypeError {}
