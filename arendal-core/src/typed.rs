use super::Integer;
use crate::ast::{BinaryOp, UnaryOp};
use crate::error::Loc;
use crate::symbol::{Path, Symbol};
use crate::types::Type;
use crate::value::Value;
use std::fmt;
use std::slice::Iter;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq)]
pub struct Expression {
    pub loc: Loc,
    pub tipo: Type,
    pub expr: Expr,
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} : {:?}", self.expr, self.tipo)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Unary {
    pub op: UnaryOp,
    pub expr: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Binary {
    pub op: BinaryOp,
    pub expr1: Expression,
    pub expr2: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    pub symbol: Symbol,
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Value(Value),
    LocalSymbol(Symbol),
    Assignment(Arc<Assignment>),
    Unary(Arc<Unary>),
    Binary(Arc<Binary>),
}

pub struct ExprBuilder {
    loc: Loc,
}

impl ExprBuilder {
    pub const fn new(loc: Loc) -> Self {
        ExprBuilder { loc }
    }

    fn build(&self, tipo: Type, expr: Expr) -> Expression {
        Expression {
            loc: self.loc.clone(),
            tipo,
            expr,
        }
    }

    pub fn value(&self, value: Value) -> Expression {
        self.build(value.clone_type(), Expr::Value(value))
    }

    pub fn val_integer(&self, value: Integer) -> Expression {
        self.value(Value::Integer(value))
    }

    pub fn val_i64(&self, value: i64) -> Expression {
        self.val_integer(value.into())
    }

    pub fn val(&self, id: Symbol, tipo: Type) -> Expression {
        self.build(tipo, Expr::LocalSymbol(id))
    }

    pub fn assignment(&self, symbol: Symbol, expr: Expression) -> Expression {
        self.build(
            expr.tipo.clone(),
            Expr::Assignment(Arc::new(Assignment { symbol, expr })),
        )
    }

    pub fn unary(&self, tipo: Type, op: UnaryOp, expr: Expression) -> Expression {
        self.build(tipo, Expr::Unary(Arc::new(Unary { op, expr })))
    }

    pub fn binary(
        &self,
        tipo: Type,
        op: BinaryOp,
        expr1: Expression,
        expr2: Expression,
    ) -> Expression {
        self.build(tipo, Expr::Binary(Arc::new(Binary { op, expr1, expr2 })))
    }

    pub fn add(&self, tipo: Type, expr1: Expression, expr2: Expression) -> Expression {
        self.binary(tipo, BinaryOp::Add, expr1, expr2)
    }

    pub fn add_i64(&self, value1: i64, value2: i64) -> Expression {
        self.add(Type::Integer, self.val_i64(value1), self.val_i64(value2))
    }

    pub fn sub(&self, tipo: Type, expr1: Expression, expr2: Expression) -> Expression {
        self.binary(tipo, BinaryOp::Sub, expr1, expr2)
    }

    pub fn sub_i64(&self, value1: i64, value2: i64) -> Expression {
        self.sub(Type::Integer, self.val_i64(value1), self.val_i64(value2))
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
