use std::fmt::Debug;

use crate::position::EqNoPosition;

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

#[derive(Debug)]
pub struct Seq<E: EqNoPosition + Debug> {
    pub expr: E,
    pub then: E,
}

impl<E: EqNoPosition + Debug> EqNoPosition for Seq<E> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.expr.eq_nopos(&other.expr) && self.then.eq_nopos(&other.then)
    }
}

#[derive(Debug)]
pub struct Conditional<E: EqNoPosition + Debug> {
    pub expr: E,
    pub then: E,
    pub otherwise: E,
}

impl<E: EqNoPosition + Debug> EqNoPosition for Conditional<E> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.expr.eq_nopos(&other.expr)
            && self.then.eq_nopos(&other.then)
            && self.otherwise.eq_nopos(&other.otherwise)
    }
}
