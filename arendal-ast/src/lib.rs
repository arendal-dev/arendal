pub mod error;
pub mod loc;
pub mod typed;
pub mod types;

pub use arcstr::{literal, ArcStr, Substr};
pub use loc::Loc;
pub use typed::{TExpr, TypedExpr};
pub use types::Type;

use num::Integer;
use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

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

#[derive(Debug)]
struct Inner {
    loc: Loc,
    expr: Expr,
}

#[derive(Clone)]
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

#[derive(Debug)]
pub enum Expr {
    LitInteger(Integer),
    Unary(UnaryOp, Expression),
    Binary(BinaryOp, Expression, Expression),
}

pub mod helper {
    use super::{BinaryOp, Expression, Integer, Loc, UnaryOp};

    pub fn lit_integer(value: Integer) -> Expression {
        Expression::lit_integer(Loc::none(), value)
    }

    pub fn lit_i64(value: i64) -> Expression {
        lit_integer(value.into())
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
