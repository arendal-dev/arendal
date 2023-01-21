pub mod bare;
pub mod error;

pub use num::bigint::{BigInt, ToBigInt};

pub trait Loc {}

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

    fn to_bare(&self) -> bare::Expression {
        match &self.expr {
            Expr::IntLiteral(value) => bare::int_literal(value.clone()),
            Expr::Unary(op, e) => bare::unary(*op, e.to_bare()),
            Expr::Binary(op, e1, e2) => bare::binary(*op, e1.to_bare(), e2.to_bare()),
        }
    }

    pub fn int_literal(loc: L, value: BigInt) -> Self {
        Self::new(loc, Expr::IntLiteral(value))
    }

    pub fn unary(loc: L, op: UnaryOp, expr: Expression<L>) -> Self {
        Self::new(loc, Expr::Unary(op, Box::new(expr)))
    }

    pub fn binary(loc: L, op: BinaryOp, expr1: Expression<L>, expr2: Expression<L>) -> Self {
        Self::new(loc, Expr::Binary(op, Box::new(expr1), Box::new(expr2)))
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
