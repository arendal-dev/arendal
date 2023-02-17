pub mod bare;
pub mod error;
pub mod types;

pub use arcstr::{literal, ArcStr, Substr};
pub use types::Type;

use num::Integer;
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;
use std::rc::Rc;

// Object-safe part of the loc trait
pub trait SafeLoc: Debug {}

// Loc isn't object safe, as Clone requires Sized
pub trait Loc: SafeLoc + Clone {}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression<P> {
    inner: Rc<InnerExpr<P>>,
}

impl<P> Expression<P> {
    fn new(payload: P, expr: Expr<P>) -> Self {
        Expression {
            inner: Rc::new(InnerExpr::new(payload, expr)),
        }
    }

    pub fn borrow_expr(&self) -> &Expr<P> {
        &self.inner.expr
    }

    pub fn borrow_payload(&self) -> &P {
        &self.inner.payload
    }

    pub fn to_bare(&self) -> bare::Expression {
        match &self.inner.expr {
            Expr::LitInteger(value) => bare::lit_integer(value.clone()),
            Expr::Unary(op, e) => bare::unary(*op, e.to_bare()),
            Expr::Binary(op, e1, e2) => bare::binary(*op, e1.to_bare(), e2.to_bare()),
        }
    }

    pub fn lit_integer(payload: P, value: Integer) -> Self {
        Self::new(payload, Expr::LitInteger(value))
    }

    pub fn unary(payload: P, op: UnaryOp, expr: Expression<P>) -> Self {
        Self::new(payload, Expr::Unary(op, expr))
    }

    pub fn binary(payload: P, op: BinaryOp, expr1: Expression<P>, expr2: Expression<P>) -> Self {
        Self::new(payload, Expr::Binary(op, expr1, expr2))
    }
}

impl<P: Clone> Expression<P> {
    pub fn clone_payload(&self) -> P {
        self.inner.payload.clone()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct InnerExpr<P> {
    payload: P,
    expr: Expr<P>,
}

impl<P> InnerExpr<P> {
    fn new(payload: P, expr: Expr<P>) -> Self {
        InnerExpr { payload, expr }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<P> {
    LitInteger(Integer),
    Unary(UnaryOp, Expression<P>),
    Binary(BinaryOp, Expression<P>, Expression<P>),
}

pub struct TypedLoc<L: SafeLoc> {
    pub loc: L,
    pub loc_type: Type,
}

impl<L: Loc> TypedLoc<L> {
    pub fn new(expr: &Expression<L>, loc_type: Type) -> Self {
        TypedLoc {
            loc: expr.clone_payload(),
            loc_type,
        }
    }
}

pub type TypedExpression<L> = Expression<TypedLoc<L>>;
