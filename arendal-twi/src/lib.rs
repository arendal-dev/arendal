mod expr;
mod value;

use ast::error::{Error, Result};
use ast::{Loc, SafeLoc, Type, TypedExpression};
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
pub struct RuntimeError {
    loc: Box<dyn SafeLoc>,
}

impl RuntimeError {
    fn new<L: Loc + 'static>(loc: L) -> Self {
        RuntimeError { loc: Box::new(loc) }
    }
}

impl Error for RuntimeError {}

pub type ValueResult = Result<TypedValue>;

pub fn expression<L: Loc + 'static>(expr: TypedExpression<L>) -> ValueResult {
    expr::eval(expr)
}
