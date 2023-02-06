pub mod bare;
pub mod error;

pub use arcstr::{literal, ArcStr, Substr};
pub use arendal_num::{Decimal, Integer};

use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;

pub trait Loc: Debug + PartialEq + Eq {}

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
}

#[derive(Debug, PartialEq, Eq)]
pub struct Expression<P> {
    payload: P,
    expr: Expr<P>,
}

impl<P> Expression<P> {
    fn new(payload: P, expr: Expr<P>) -> Self {
        Expression { payload, expr }
    }

    pub fn to_bare(&self) -> bare::Expression {
        match &self.expr {
            Expr::LitInteger(value) => bare::lit_integer(value.clone()),
            Expr::Unary(op, e) => bare::unary(*op, e.to_bare()),
            Expr::Binary(op, e1, e2) => bare::binary(*op, e1.to_bare(), e2.to_bare()),
        }
    }

    pub fn lit_integer(payload: P, value: Integer) -> Self {
        Self::new(payload, Expr::LitInteger(value))
    }

    pub fn unary(payload: P, op: UnaryOp, expr: Expression<P>) -> Self {
        Self::new(payload, Expr::Unary(op, Box::new(expr)))
    }

    pub fn binary(payload: P, op: BinaryOp, expr1: Expression<P>, expr2: Expression<P>) -> Self {
        Self::new(payload, Expr::Binary(op, Box::new(expr1), Box::new(expr2)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<L> {
    LitInteger(Integer),
    Unary(UnaryOp, Box<Expression<L>>),
    Binary(BinaryOp, Box<Expression<L>>, Box<Expression<L>>),
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
