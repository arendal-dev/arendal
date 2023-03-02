use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use super::Integer;
use crate::error::Loc;
use crate::id::{Identifier, TypeIdentifier};

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
struct Inner {
    loc: Loc,
    expr: Expr,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Expression {
    inner: Rc<Inner>,
}

impl Expression {
    fn new(loc: Loc, expr: Expr) -> Self {
        Expression {
            inner: Rc::new(Inner { loc, expr }),
        }
    }

    pub fn borrow_loc(&self) -> &Loc {
        &self.inner.loc
    }

    pub fn borrow_expr(&self) -> &Expr {
        &self.inner.expr
    }

    pub fn lit_integer(loc: Loc, value: Integer) -> Self {
        Self::new(loc, Expr::LitInteger(value))
    }

    pub fn lit_type(loc: Loc, id: TypeIdentifier) -> Self {
        Self::new(loc, Expr::LitType(id))
    }

    pub fn unary(loc: Loc, op: UnaryOp, expr: Expression) -> Self {
        Self::new(loc, Expr::Unary(op, expr))
    }

    pub fn binary(loc: Loc, op: BinaryOp, expr1: Expression, expr2: Expression) -> Self {
        Self::new(loc, Expr::Binary(op, expr1, expr2))
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expr.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    LitInteger(Integer),
    LitType(TypeIdentifier),
    Id(Identifier),
    Unary(UnaryOp, Expression),
    Binary(BinaryOp, Expression, Expression),
}

pub mod helper {
    use super::{BinaryOp, Expression, Integer, Loc, TypeIdentifier, UnaryOp};

    pub fn lit_integer(value: Integer) -> Expression {
        Expression::lit_integer(Loc::none(), value)
    }

    pub fn lit_i64(value: i64) -> Expression {
        lit_integer(value.into())
    }

    pub fn lit_type(id: TypeIdentifier) -> Expression {
        Expression::lit_type(Loc::none(), id)
    }

    pub fn lit_type_str(id: &str) -> Expression {
        lit_type(TypeIdentifier::new(id).unwrap())
    }

    pub fn unary(op: UnaryOp, expr: Expression) -> Expression {
        Expression::unary(Loc::none(), op, expr)
    }

    pub fn binary(op: BinaryOp, expr1: Expression, expr2: Expression) -> Expression {
        Expression::binary(Loc::none(), op, expr1, expr2)
    }

    pub fn add(expr1: Expression, expr2: Expression) -> Expression {
        binary(BinaryOp::Add, expr1, expr2)
    }

    pub fn add_i64(value1: i64, value2: i64) -> Expression {
        add(lit_i64(value1), lit_i64(value2))
    }

    pub fn sub(expr1: Expression, expr2: Expression) -> Expression {
        binary(BinaryOp::Sub, expr1, expr2)
    }

    pub fn sub_i64(value1: i64, value2: i64) -> Expression {
        sub(lit_i64(value1), lit_i64(value2))
    }
}
