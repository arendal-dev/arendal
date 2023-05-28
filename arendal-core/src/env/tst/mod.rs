mod twi;
mod typecheck;

use im::HashMap;

use crate::ast::UnaryOp;
use crate::error::{Error, Loc, Result, L};
use crate::symbol::{FQType, Pkg, Symbol};
use crate::types::Type;
use crate::visibility::Visibility;
use crate::Integer;
use std::fmt;
use std::sync::Arc;

use super::{Env, Value};

pub(super) fn run(env: &mut Env, input: &str) -> Result<Value> {
    let package = crate::ast::parser::parse(input)?;
    let checked = typecheck::check(&env, &package)?;
    twi::interpret(env, &checked)
}

impl L<Expr> {
    fn borrow_type(&self) -> &Type {
        self.it.borrow_type()
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

    fn type_mismatch<T>(&self, expected: Type) -> Result<T> {
        self.err(Error::type_mismatch(expected, self.clone_type()))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Unary {
    op: UnaryOp,
    expr: L<Expr>,
}

#[derive(Debug, PartialEq, Eq)]
struct TwoInts {
    expr1: L<Expr>,
    expr2: L<Expr>,
}

impl TwoInts {
    fn new(expr1: L<Expr>, expr2: L<Expr>) -> Result<Arc<TwoInts>> {
        Error::merge(expr1.check_integer(), expr2.check_integer())?;
        Ok(Arc::new(TwoInts { expr1, expr2 }))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TwoBools {
    expr1: L<Expr>,
    expr2: L<Expr>,
}

impl TwoBools {
    fn new(expr1: L<Expr>, expr2: L<Expr>) -> Result<Arc<TwoBools>> {
        Error::merge(expr1.check_boolean(), expr2.check_boolean())?;
        Ok(Arc::new(TwoBools { expr1, expr2 }))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Conditional {
    expr: L<Expr>,
    then: L<Expr>,
    otherwise: L<Expr>,
}

impl Conditional {
    fn new(expr: L<Expr>, then: L<Expr>, otherwise: L<Expr>) -> Result<Arc<Self>> {
        Error::merge(expr.check_boolean(), Self::same_types(&then, &otherwise))?;
        Ok(Arc::new(Self {
            expr,
            then,
            otherwise,
        }))
    }

    fn same_types(then: &L<Expr>, otherwise: &L<Expr>) -> Result<()> {
        if then.borrow_type() == otherwise.borrow_type() {
            Ok(())
        } else {
            otherwise.type_mismatch(then.clone_type())
        }
    }

    fn borrow_type(&self) -> &Type {
        self.then.borrow_type()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Assignment {
    symbol: Symbol,
    expr: L<Expr>,
}

#[derive(Debug, PartialEq, Eq)]
struct Local {
    symbol: Symbol,
    tipo: Type,
}

#[derive(Clone, PartialEq, Eq)]
enum Expr {
    Value(Value),
    Local(Arc<Local>),
    Conditional(Arc<Conditional>),
    Assignment(Arc<Assignment>),
    Unary(Arc<Unary>),
    IntAdd(Arc<TwoInts>),
    IntSub(Arc<TwoInts>),
    IntMul(Arc<TwoInts>),
    IntDiv(Arc<TwoInts>),
    LogicalAnd(Arc<TwoBools>),
    LogicalOr(Arc<TwoBools>),
    Block(Vec<L<Expr>>),
}

impl Expr {
    pub(crate) fn borrow_type(&self) -> &Type {
        match self {
            Self::Value(v) => v.borrow_type(),
            Self::Local(l) => &l.tipo,
            Self::Conditional(c) => c.borrow_type(),
            Self::Assignment(a) => a.expr.borrow_type(),
            Self::Unary(u) => u.expr.borrow_type(),
            Self::IntAdd(t) => t.expr1.borrow_type(),
            Self::IntSub(t) => t.expr1.borrow_type(),
            Self::IntMul(t) => t.expr1.borrow_type(),
            Self::IntDiv(t) => t.expr1.borrow_type(),
            Self::LogicalAnd(_) | Self::LogicalOr(_) => &Type::Boolean,
            Self::Block(exprs) => {
                if exprs.is_empty() {
                    &Type::None
                } else {
                    exprs.last().unwrap().borrow_type()
                }
            }
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} : {:?}", self, self.borrow_type())
    }
}

struct ExprBuilder {
    loc: Loc,
}

impl ExprBuilder {
    const fn new(loc: Loc) -> Self {
        ExprBuilder { loc }
    }

    fn build(&self, expr: Expr) -> L<Expr> {
        self.loc.wrap(expr)
    }

    fn ok(&self, expr: Expr) -> Result<L<Expr>> {
        Ok(self.build(expr))
    }

    fn value(&self, value: Value) -> L<Expr> {
        self.build(Expr::Value(value))
    }

    fn v_none(&self) -> L<Expr> {
        self.value(Value::None)
    }

    fn val_integer(&self, value: Integer) -> L<Expr> {
        self.value(Value::Integer(value))
    }

    fn local(&self, symbol: Symbol, tipo: Type) -> L<Expr> {
        self.build(Expr::Local(Arc::new(Local { symbol, tipo })))
    }

    fn conditional(&self, expr: L<Expr>, then: L<Expr>, otherwise: L<Expr>) -> Result<L<Expr>> {
        self.ok(Expr::Conditional(Conditional::new(expr, then, otherwise)?))
    }

    fn assignment(&self, symbol: Symbol, expr: L<Expr>) -> L<Expr> {
        self.build(Expr::Assignment(Arc::new(Assignment { symbol, expr })))
    }

    fn unary(&self, op: UnaryOp, expr: L<Expr>) -> L<Expr> {
        self.build(Expr::Unary(Arc::new(Unary { op, expr })))
    }

    fn int_add(&self, expr1: L<Expr>, expr2: L<Expr>) -> Result<L<Expr>> {
        self.ok(Expr::IntAdd(TwoInts::new(expr1, expr2)?))
    }

    fn int_sub(&self, expr1: L<Expr>, expr2: L<Expr>) -> Result<L<Expr>> {
        self.ok(Expr::IntSub(TwoInts::new(expr1, expr2)?))
    }

    fn int_mul(&self, expr1: L<Expr>, expr2: L<Expr>) -> Result<L<Expr>> {
        self.ok(Expr::IntMul(TwoInts::new(expr1, expr2)?))
    }

    fn int_div(&self, expr1: L<Expr>, expr2: L<Expr>) -> Result<L<Expr>> {
        self.ok(Expr::IntDiv(TwoInts::new(expr1, expr2)?))
    }

    fn log_and(&self, expr1: L<Expr>, expr2: L<Expr>) -> Result<L<Expr>> {
        self.ok(Expr::LogicalAnd(TwoBools::new(expr1, expr2)?))
    }

    fn log_or(&self, expr1: L<Expr>, expr2: L<Expr>) -> Result<L<Expr>> {
        self.ok(Expr::LogicalOr(TwoBools::new(expr1, expr2)?))
    }

    fn block(&self, exprs: Vec<L<Expr>>) -> Result<L<Expr>> {
        if exprs.is_empty() {
            Ok(self.v_none())
        } else {
            self.ok(Expr::Block(exprs))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDefinition {
    loc: Loc,
    visibility: Visibility,
    tipo: Type,
}

type TypeDefMap = HashMap<FQType, TypeDefinition>;

#[derive(Debug)]
pub(super) struct Package {
    pkg: Pkg,
    types: TypeDefMap,
    expressions: Vec<L<Expr>>,
}
