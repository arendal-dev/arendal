mod value;

use ast::error::{Error, Errors, Result};
use ast::{Loc, Type, TypedExpression};
use std::fmt;

pub use value::Value;

#[derive(Debug, PartialEq, Eq)]
pub struct TypedValue {
    pub value: Value,
    pub value_type: Type,
}

impl TypedValue {
    fn new(value: Value, value_type: Type) -> Self {
        TypedValue { value, value_type }
    }

    pub fn integer(value: num::Integer) -> Self {
        Self::new(Value::integer(value), Type::Integer)
    }

    pub fn int64(value: i64) -> Self {
        Self::integer(value.into())
    }
}

impl fmt::Display for TypedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} : {}", self.value, self.value_type)
    }
}

#[derive(Debug)]
pub struct RuntimeError<L: Loc> {
    loc: L,
}

impl<L: Loc> Error for RuntimeError<L> {}

fn ok(value: Value, value_type: Type) -> Result<TypedValue> {
    Ok(TypedValue::new(value, value_type))
}

fn err<L: Loc + 'static>(expr: &TypedExpression<L>) -> Result<TypedValue> {
    let mut errors: Errors = Default::default();
    errors.add(RuntimeError {
        loc: expr.payload.loc.clone(),
    });
    Err(errors)
}

pub fn expression<L: Loc + 'static>(expr: &TypedExpression<L>) -> Result<TypedValue> {
    match &expr.expr {
        ast::Expr::LitInteger(i) => ok(Value::Integer(i.clone()), expr.payload.loc_type.clone()),
        _ => err(&expr),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
