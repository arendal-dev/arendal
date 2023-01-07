pub mod error;

use num::bigint::BigInt;

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

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    IntLiteral(BigInt),
    Unary(UnaryOp, Box<Expression>),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
