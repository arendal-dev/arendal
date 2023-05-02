mod twi;
mod typecheck;

use crate::ast::UnaryOp;
use crate::error::{Error, Loc, Result};
use crate::symbol::{Path, Symbol};
use crate::types::Type;
use crate::value::Value;
use crate::Integer;
use std::fmt;
use std::slice::Iter;
use std::sync::Arc;

use super::Env;

pub(super) fn run(env: &mut Env, path: &Path, input: &str) -> Result<Value> {
    let parsed = crate::parser::parse(input)?;
    let checked = typecheck::check(&env, &path, &parsed)?;
    twi::interpret(env, &checked)
}

#[derive(Clone, PartialEq, Eq)]
struct Expression {
    loc: Loc,
    expr: Expr,
}

impl Expression {
    fn borrow_type(&self) -> &Type {
        self.expr.borrow_type()
    }

    fn clone_type(&self) -> Type {
        self.borrow_type().clone()
    }

    fn check_integer(&self) -> Result<()> {
        if self.borrow_type().is_integer() {
            Ok(())
        } else {
            self.type_mismatch(Type::Integer)
        }
    }

    fn check_boolean(&self) -> Result<()> {
        if self.borrow_type().is_boolean() {
            Ok(())
        } else {
            self.type_mismatch(Type::Boolean)
        }
    }

    fn err<T>(&self, error: Error) -> Result<T> {
        self.loc.err(error)
    }

    fn type_mismatch<T>(&self, expected: Type) -> Result<T> {
        self.err(Error::type_mismatch(expected, self.clone_type()))
    }

    fn rt_err<T>(&self, error: Error) -> Result<T> {
        self.loc.err(error)
    }

    fn rt_type_mismatch<T>(&self, expected: Type) -> Result<T> {
        self.rt_err(Error::type_mismatch(expected, self.clone_type()))
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} : {:?}", self.expr, self.borrow_type())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Unary {
    op: UnaryOp,
    expr: Expression,
}

#[derive(Debug, PartialEq, Eq)]
struct Two {
    expr1: Expression,
    expr2: Expression,
}

impl Two {
    fn new(expr1: Expression, expr2: Expression) -> Arc<Two> {
        Arc::new(Two { expr1, expr2 })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TwoInts {
    expr1: Expression,
    expr2: Expression,
}

impl TwoInts {
    fn new(expr1: Expression, expr2: Expression) -> Result<Arc<TwoInts>> {
        Error::merge(expr1.check_integer(), expr2.check_integer())?;
        Ok(Arc::new(TwoInts { expr1, expr2 }))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TwoBools {
    expr1: Expression,
    expr2: Expression,
}

impl TwoBools {
    fn new(expr1: Expression, expr2: Expression) -> Result<Arc<TwoBools>> {
        Error::merge(expr1.check_boolean(), expr2.check_boolean())?;
        Ok(Arc::new(TwoBools { expr1, expr2 }))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Assignment {
    symbol: Symbol,
    expr: Expression,
}

#[derive(Debug, PartialEq, Eq)]
struct Local {
    symbol: Symbol,
    tipo: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Value(Value),
    Local(Arc<Local>),
    Assignment(Arc<Assignment>),
    Unary(Arc<Unary>),
    Add(Arc<Two>),
    Sub(Arc<Two>),
    Mul(Arc<Two>),
    Div(Arc<Two>),
    LogicalAnd(Arc<Two>),
    LogicalOr(Arc<Two>),
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
            Self::LogicalAnd(_) | Self::LogicalOr(_) => &Type::Boolean,
        }
    }
}

struct ExprBuilder {
    loc: Loc,
}

impl ExprBuilder {
    const fn new(loc: Loc) -> Self {
        ExprBuilder { loc }
    }

    fn build(&self, expr: Expr) -> Expression {
        Expression {
            loc: self.loc.clone(),
            expr,
        }
    }

    fn value(&self, value: Value) -> Expression {
        self.build(Expr::Value(value))
    }

    fn val_integer(&self, value: Integer) -> Expression {
        self.value(Value::integer(&self.loc, value))
    }

    fn local(&self, symbol: Symbol, tipo: Type) -> Expression {
        self.build(Expr::Local(Arc::new(Local { symbol, tipo })))
    }

    fn assignment(&self, symbol: Symbol, expr: Expression) -> Expression {
        self.build(Expr::Assignment(Arc::new(Assignment { symbol, expr })))
    }

    fn unary(&self, op: UnaryOp, expr: Expression) -> Expression {
        self.build(Expr::Unary(Arc::new(Unary { op, expr })))
    }

    fn add(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::Add(Two::new(expr1, expr2)))
    }

    fn sub(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::Sub(Two::new(expr1, expr2)))
    }

    fn mul(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::Mul(Two::new(expr1, expr2)))
    }

    fn div(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::Div(Two::new(expr1, expr2)))
    }

    fn log_and(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::LogicalAnd(Two::new(expr1, expr2)))
    }

    fn log_or(&self, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::LogicalOr(Two::new(expr1, expr2)))
    }
}

#[derive(Debug)]
struct Expressions {
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
pub(super) struct Module {
    path: Path,
    expressions: Expressions,
}
