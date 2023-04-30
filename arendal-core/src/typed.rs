use super::Integer;
use crate::ast::UnaryOp;
use crate::error::Loc;
use crate::symbol::{Path, Symbol};
use crate::types::Type;
use crate::value::Value;
use std::fmt;
use std::slice::Iter;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq)]
pub struct Expression {
    pub(crate) loc: Loc,
    pub(crate) expr: Expr,
}

impl Expression {
    pub(crate) fn borrow_type(&self) -> &Type {
        self.expr.borrow_type()
    }

    pub(crate) fn clone_type(&self) -> Type {
        self.borrow_type().clone()
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} : {:?}", self.expr, self.borrow_type())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Unary {
    pub(crate) op: UnaryOp,
    pub(crate) expr: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Two {
    pub(crate) expr1: Expression,
    pub(crate) expr2: Expression,
}

impl Two {
    fn new(expr1: Expression, expr2: Expression) -> Arc<Two> {
        Arc::new(Two { expr1, expr2 })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Assignment {
    pub(crate) symbol: Symbol,
    pub(crate) expr: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Local {
    pub(crate) symbol: Symbol,
    pub(crate) tipo: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Expr {
    Value(Value),
    Local(Arc<Local>),
    Assignment(Arc<Assignment>),
    Unary(Arc<Unary>),
    Add(Arc<Two>),
    Sub(Arc<Two>),
    Mul(Arc<Two>),
    Div(Arc<Two>),
}

impl Expr {
    pub(crate) fn borrow_type(&self) -> &Type {
        match self {
            Self::Value(v) => v.borrow_type(),
            Self::Local(l) => &l.tipo,
            Self::Assignment(a) => a.expr.borrow_type(),
            Self::Unary(u) => u.expr.borrow_type(),
            Self::Add(t) => t.expr1.borrow_type(),
            Self::Sub(t) => t.expr1.borrow_type(),
            Self::Mul(t) => t.expr1.borrow_type(),
            Self::Div(t) => t.expr1.borrow_type(),
        }
    }
}

pub(crate) struct ExprBuilder {
    loc: Loc,
}

impl ExprBuilder {
    pub(crate) const fn new(loc: Loc) -> Self {
        ExprBuilder { loc }
    }

    fn build(&self, expr: Expr) -> Expression {
        Expression {
            loc: self.loc.clone(),
            expr,
        }
    }

    pub(crate) fn value(&self, value: Value) -> Expression {
        self.build(Expr::Value(value))
    }

    pub(crate) fn val_integer(&self, value: Integer) -> Expression {
        self.value(Value::Integer(value))
    }

    pub(crate) fn local(&self, symbol: Symbol, tipo: Type) -> Expression {
        self.build(Expr::Local(Arc::new(Local { symbol, tipo })))
    }

    pub(crate) fn assignment(&self, symbol: Symbol, expr: Expression) -> Expression {
        self.build(Expr::Assignment(Arc::new(Assignment { symbol, expr })))
    }

    pub(crate) fn unary(&self, op: UnaryOp, expr: Expression) -> Expression {
        self.build(Expr::Unary(Arc::new(Unary { op, expr })))
    }

    pub(crate) fn add(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::Add(Two::new(expr1, expr2)))
    }

    pub(crate) fn sub(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::Sub(Two::new(expr1, expr2)))
    }

    pub(crate) fn mul(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::Mul(Two::new(expr1, expr2)))
    }

    pub(crate) fn div(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::Mul(Two::new(expr1, expr2)))
    }
}

#[derive(Debug)]
pub struct Expressions {
    expressions: Vec<Expression>,
}

impl Expressions {
    pub fn new(expressions: Vec<Expression>) -> Self {
        Self { expressions }
    }

    pub fn iter(&self) -> Iter<'_, Expression> {
        self.expressions.iter()
    }
}

impl<'a> IntoIterator for &'a Expressions {
    type Item = &'a Expression;
    type IntoIter = Iter<'a, Expression>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct Module {
    pub path: Path,
    pub expressions: Expressions,
}
