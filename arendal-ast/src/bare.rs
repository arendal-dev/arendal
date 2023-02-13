use super::{BinaryOp, Integer, UnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BareLoc {}

impl super::Loc for BareLoc {}

pub type Expression = super::Expression<BareLoc>;

pub fn lit_integer(value: Integer) -> Expression {
    super::Expression::lit_integer(BareLoc {}, value)
}

pub fn lit_i64(value: i64) -> Expression {
    lit_integer(value.into())
}

pub fn unary(op: UnaryOp, expr: Expression) -> Expression {
    super::Expression::unary(BareLoc {}, op, expr)
}

pub fn binary(op: BinaryOp, expr1: Expression, expr2: Expression) -> Expression {
    super::Expression::binary(BareLoc {}, op, expr1, expr2)
}

pub fn add(left: Expression, right: Expression) -> Expression {
    binary(BinaryOp::Add, left, right)
}

pub fn add_i64(v1: i64, v2: i64) -> Expression {
    add(lit_i64(v1), lit_i64(v2))
}
