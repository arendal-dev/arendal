use std::collections::HashMap;

use ast::{position::Position, symbol::FQType};
use num::Integer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    Unit,
    True,
    False,
    Integer, // Temporary
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TypeExpr {
    Type(Type),
}

#[derive(Debug)]
pub(crate) struct TypeDfn {
    position: Position,
    type_expr: TypeExpr,
}

#[derive(Debug, Default)]
pub(crate) struct Types {
    values: HashMap<FQType, TypeDfn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Unit,
    True,
    False,
    Integer(Integer),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedValue {
    value: Value,
    value_type: Type,
}
