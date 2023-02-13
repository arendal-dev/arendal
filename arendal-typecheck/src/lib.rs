mod expr;

use ast::error::{Error, Errors, Result};
use ast::{Expression, Loc, Type, TypedExpression, TypedLoc};

// 'static here means that L is owned
pub fn expression<L: Loc + 'static>(input: &Expression<L>) -> Result<TypedExpression<L>> {
    expr::check(input)
}

#[derive(Debug)]
struct TypeError<L: Loc> {
    loc: L,
    kind: TypeErrorKind,
}

impl<L: Loc> TypeError<L> {
    fn new(expr: &Expression<L>, kind: TypeErrorKind) -> Self {
        TypeError {
            loc: expr.payload.clone(),
            kind,
        }
    }
}

#[derive(Debug)]
enum TypeErrorKind {
    InvalidType, // placeholder, temporary error
}

impl<L: Loc> Error for TypeError<L> {}
