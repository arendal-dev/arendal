use crate::ast::Expression;
use crate::error::{Error, Result};
use crate::identifier::Id;
use crate::names::Names;
use crate::typed::TypedExpr;

mod expr;

pub fn expression(names: &mut Names, input: &Expression) -> Result<TypedExpr> {
    expr::check(names, input)
}

#[derive(Debug)]
enum TypeError {
    UnknownIdentifier(Id),
    InvalidType, // placeholder, temporary error
}

impl Error for TypeError {}
