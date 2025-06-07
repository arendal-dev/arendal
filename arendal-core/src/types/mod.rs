use crate::itr::Payload;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Type {
    Unit,
    True,
    False,
    Integer, // Temporary
}

impl Payload for Type {}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TypeExpr {
    Type(Type),
}

impl Payload for TypeExpr {}
