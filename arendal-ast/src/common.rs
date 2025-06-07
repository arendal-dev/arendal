use std::fmt::{self, Debug};

use crate::position::{EqNoPosition, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NEq,
    GT,
    GE,
    LT,
    LE,
    And,
    Or,
}

#[derive(Debug)]
pub struct Unary<E: EqNoPosition + Debug> {
    pub op: UnaryOp,
    pub expr: E,
}

impl<E: EqNoPosition + Debug> EqNoPosition for Unary<E> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.op == other.op && self.expr.eq_nopos(&other.expr)
    }
}

#[derive(Debug)]
pub struct Binary<E: EqNoPosition + Debug> {
    pub op: BinaryOp,
    pub expr1: E,
    pub expr2: E,
}

impl<E: EqNoPosition + Debug> EqNoPosition for Binary<E> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.op == other.op
            && self.expr1.eq_nopos(&other.expr1)
            && self.expr2.eq_nopos(&other.expr2)
    }
}
