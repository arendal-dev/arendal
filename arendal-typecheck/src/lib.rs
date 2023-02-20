mod expr;

use ast::error::{Error, Result};
use ast::{Expression, Loc, TExpr, TypedExpr};

// 'static here means that L is owned
pub fn expression(input: Expression) -> Result<TypedExpr> {
    expr::check(input)
}

#[derive(Debug)]
struct TypeError {
    loc: Loc,
    kind: TypeErrorKind,
}

impl TypeError {
    fn new(loc: Loc, kind: TypeErrorKind) -> Self {
        TypeError { loc, kind }
    }
}

#[derive(Debug)]
enum TypeErrorKind {
    InvalidType, // placeholder, temporary error
}

impl Error for TypeError {}
