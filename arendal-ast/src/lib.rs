pub mod error;

use num::bigint::BigInt;
use std::marker;

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
pub struct Expression<'a, L: error::ErrorLoc> {
    loc: L,
    expr: Expr<'a, L>,
    _marker: marker::PhantomData<&'a L>,
}

impl<'a, L: error::ErrorLoc> Expression<'a, L> {
    fn new(loc: L, expr: Expr<'a, L>) -> Self {
        Expression {
            loc,
            expr,
            _marker: marker::PhantomData,
        }
    }

    fn int_literal(loc: L, value: BigInt) -> Self {
        Self::new(loc, Expr::IntLiteral(value))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'a, L: error::ErrorLoc> {
    IntLiteral(BigInt),
    Unary(UnaryOp, Box<Expression<'a, L>>),
    Binary(BinaryOp, Box<Expression<'a, L>>, Box<Expression<'a, L>>),
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
