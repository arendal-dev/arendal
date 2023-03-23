use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use super::Integer;
use crate::error::Loc;
use crate::symbol::{Symbol, TSymbol};

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

    pub fn clone_loc(&self) -> Loc {
        self.inner.loc.clone()
    }

    pub fn borrow_expr(&self) -> &Expr {
        &self.inner.expr
    }

    pub fn clone_expr(&self) -> Expr {
        self.inner.expr.clone()
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expr.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    LitInteger(Integer),
    Symbol(Symbol),
    TSymbol(TSymbol),
    Unary(UnaryOp, Expression),
    Binary(BinaryOp, Expression, Expression),
    Block(Vec<Expression>),
    Assignment(Symbol, Expression),
}

pub struct ExprBuilder {
    loc: Loc,
}

impl ExprBuilder {
    pub const fn new(loc: Loc) -> Self {
        ExprBuilder { loc }
    }

    pub const fn none() -> Self {
        Self::new(Loc::none())
    }

    pub fn lit_integer(&self, value: Integer) -> Expression {
        Expression::new(self.loc.clone(), Expr::LitInteger(value))
    }

    pub fn lit_i64(&self, value: i64) -> Expression {
        self.lit_integer(value.into())
    }

    pub fn symbol(&self, symbol: Symbol) -> Expression {
        Expression::new(self.loc.clone(), Expr::Symbol(symbol))
    }

    pub fn tsymbol(&self, symbol: TSymbol) -> Expression {
        Expression::new(self.loc.clone(), Expr::TSymbol(symbol))
    }

    pub fn unary(&self, op: UnaryOp, expr: Expression) -> Expression {
        Expression::new(self.loc.clone(), Expr::Unary(op, expr))
    }

    pub fn binary(&self, op: BinaryOp, expr1: Expression, expr2: Expression) -> Expression {
        Expression::new(self.loc.clone(), Expr::Binary(op, expr1, expr2))
    }

    pub fn add(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.binary(BinaryOp::Add, expr1, expr2)
    }

    pub fn add_i64(&self, value1: i64, value2: i64) -> Expression {
        self.add(self.lit_i64(value1), self.lit_i64(value2))
    }

    pub fn sub(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.binary(BinaryOp::Sub, expr1, expr2)
    }

    pub fn sub_i64(&self, value1: i64, value2: i64) -> Expression {
        self.sub(self.lit_i64(value1), self.lit_i64(value2))
    }

    pub fn block(&self, mut exprs: Vec<Expression>) -> Expression {
        assert!(
            !exprs.is_empty(),
            "Blocks need to contain at least one expression"
        );
        if exprs.len() == 1 {
            exprs.pop().unwrap()
        } else {
            Expression::new(self.loc.clone(), Expr::Block(exprs))
        }
    }

    pub fn assignment(&self, id: Symbol, expr: Expression) -> Expression {
        Expression::new(self.loc.clone(), Expr::Assignment(id, expr))
    }
}
