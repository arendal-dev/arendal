pub mod error;

pub use num::bigint::{BigInt, ToBigInt};

pub trait Loc {}

#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NEq,
}

// We have a lifetime parameter as we expect locations to be a reference to some input
// which will require a lifetime.
#[derive(Debug, PartialEq, Eq)]
pub struct Expression<L: Loc> {
    loc: L,
    expr: Expr<L>,
}

impl<L: Loc> Expression<L> {
    fn new(loc: L, expr: Expr<L>) -> Self {
        Expression { loc, expr }
    }

    pub fn int_literal(loc: L, value: BigInt) -> Self {
        Self::new(loc, Expr::IntLiteral(value))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<L: Loc> {
    IntLiteral(BigInt),
    Unary(UnaryOp, Box<Expression<L>>),
    Binary(BinaryOp, Box<Expression<L>>, Box<Expression<L>>),
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
