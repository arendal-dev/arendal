mod expr;

use ast::error::{Error, Result};
use ast::{Expression, Loc, SafeLoc, TypedExpression};

// 'static here means that L is owned
pub fn expression<L: Loc + 'static>(input: Expression<L>) -> Result<TypedExpression<L>> {
    expr::check(input)
}

#[derive(Debug)]
struct TypeError {
    loc: Box<dyn SafeLoc>,
    kind: TypeErrorKind,
}

impl TypeError {
    fn new<L: Loc + 'static>(loc: L, kind: TypeErrorKind) -> Self {
        TypeError {
            loc: Box::new(loc),
            kind,
        }
    }
}

#[derive(Debug)]
enum TypeErrorKind {
    InvalidType, // placeholder, temporary error
}

impl Error for TypeError {}
