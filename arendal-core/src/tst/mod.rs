mod typecheck;

use crate::Integer;
use crate::ast0::UnaryOp;
use crate::context::{Context, Type, Value};
use crate::env0::{Env, Symbols};
use crate::error::{Error, L, Loc, Result};
use crate::symbol::{FQSym, Pkg, Symbol};
use std::fmt;
use std::sync::Arc;

pub(crate) fn check(env: &Env, input: &str) -> Result<Package> {
    let parsed = crate::ast0::parser::parse(Pkg::Local, input)?;
    typecheck::check(&env, &parsed)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Unary {
    op: UnaryOp,
    expr: L<Expr>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TwoInts {
    pub(crate) expr1: L<Expr>,
    pub(crate) expr2: L<Expr>,
}

impl TwoInts {
    fn new(expr1: L<Expr>, expr2: L<Expr>) -> Result<Arc<TwoInts>> {
        Error::merge(expr1.check_integer(), expr2.check_integer())?;
        Ok(Arc::new(TwoInts { expr1, expr2 }))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TwoBools {
    pub(crate) expr1: L<Expr>,
    pub(crate) expr2: L<Expr>,
}

impl TwoBools {
    fn new(expr1: L<Expr>, expr2: L<Expr>) -> Result<Arc<TwoBools>> {
        Error::merge(expr1.check_boolean(), expr2.check_boolean())?;
        Ok(Arc::new(TwoBools { expr1, expr2 }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seq {
    pub(crate) expr: L<Expr>,
    pub(crate) then: L<Expr>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Conditional {
    pub(crate) expr: L<Expr>,
    pub(crate) then: L<Expr>,
    pub(crate) otherwise: L<Expr>,
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
        if then.get_type() == otherwise.get_type() {
            Ok(())
        } else {
            otherwise.type_mismatch(then.get_type())
        }
    }

    fn get_type(&self) -> Type {
        self.then.get_type()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    pub(crate) symbol: Symbol,
    pub(crate) expr: L<Expr>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Local {
    pub(crate) symbol: Symbol,
    pub(crate) tipo: Type,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Global {
    pub(crate) symbol: FQSym,
    pub(crate) tipo: Type,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    pub(crate) assignments: Vec<L<Assignment>>,
    pub expr: Option<L<Expr>>,
}

impl Block {
    fn get_type(&self) -> Type {
        match &self.expr {
            Some(e) => e.get_type(),
            _ => match self.assignments.last() {
                Some(a) => a.it.expr.get_type(),
                _ => Type::type_none(),
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Expr {
    Value(Value),
    Local(Arc<Local>),
    Global(Arc<Global>),
    Seq(Arc<Seq>),
    Conditional(Arc<Conditional>),
    Unary(Arc<Unary>),
    IntAdd(Arc<TwoInts>),
    IntSub(Arc<TwoInts>),
    IntMul(Arc<TwoInts>),
    IntDiv(Arc<TwoInts>),
    LogicalAnd(Arc<TwoBools>),
    LogicalOr(Arc<TwoBools>),
    Block(Arc<Block>),
}

impl Expr {
    fn get_type(&self) -> Type {
        match self {
            Self::Value(v) => v.get_type(),
            Self::Local(l) => l.tipo.clone(),
            Self::Global(g) => g.tipo.clone(),
            Self::Seq(s) => s.then.get_type(),
            Self::Conditional(c) => c.get_type(),
            Self::Unary(u) => u.expr.get_type(),
            Self::IntAdd(t) => t.expr1.get_type(),
            Self::IntSub(t) => t.expr1.get_type(),
            Self::IntMul(t) => t.expr1.get_type(),
            Self::IntDiv(t) => t.expr1.get_type(),
            Self::LogicalAnd(_) | Self::LogicalOr(_) => Type::type_boolean(),
            Self::Block(b) => b.get_type(),
        }
    }
}

impl L<Expr> {
    pub fn get_type(&self) -> Type {
        self.it.get_type()
    }

    fn check_integer(&self) -> Result<()> {
        if self.get_type().is_integer() {
            Ok(())
        } else {
            self.type_mismatch(Type::type_integer())
        }
    }

    fn check_boolean(&self) -> Result<()> {
        if self.get_type().is_boolean() {
            Ok(())
        } else {
            self.type_mismatch(Type::type_boolean())
        }
    }

    fn type_mismatch<T>(&self, expected: Type) -> Result<T> {
        self.err(Error::type_mismatch(expected, self.get_type()))
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} : {:?}", self, self.get_type())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TLAssignment {
    pub(crate) symbol: FQSym,
    pub(crate) expr: L<Expr>,
}

struct Builder {
    loc: Loc,
}

impl Builder {
    const fn new(loc: Loc) -> Self {
        Builder { loc }
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
        self.value(Value::v_none())
    }

    fn val_integer(&self, value: Integer) -> L<Expr> {
        self.value(Value::v_integer(&self.loc, Type::type_integer(), value).unwrap())
    }

    fn local0(&self, local: Local) -> L<Expr> {
        self.build(Expr::Local(Arc::new(local)))
    }

    fn local(&self, symbol: Symbol, tipo: Type) -> L<Expr> {
        self.local0(Local { symbol, tipo })
    }

    fn global0(&self, global: Global) -> L<Expr> {
        self.build(Expr::Global(Arc::new(global)))
    }

    fn global(&self, symbol: FQSym, tipo: Type) -> L<Expr> {
        self.global0(Global { symbol, tipo })
    }

    fn seq(&self, expr: L<Expr>, then: L<Expr>) -> L<Expr> {
        self.build(Expr::Seq(Arc::new(Seq { expr, then })))
    }

    fn conditional(&self, expr: L<Expr>, then: L<Expr>, otherwise: L<Expr>) -> Result<L<Expr>> {
        self.ok(Expr::Conditional(Conditional::new(expr, then, otherwise)?))
    }

    fn assignment(&self, symbol: Symbol, expr: L<Expr>) -> L<Assignment> {
        self.loc.wrap(Assignment { symbol, expr })
    }

    fn tl_assignment(&self, symbol: FQSym, expr: L<Expr>) -> L<TLAssignment> {
        self.loc.wrap(TLAssignment { symbol, expr })
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

    fn block(&self, assignments: Vec<L<Assignment>>, expr: Option<L<Expr>>) -> Result<L<Expr>> {
        if assignments.is_empty() {
            Ok(expr.or_else(|| Some(self.v_none())).unwrap())
        } else {
            self.ok(Expr::Block(Arc::new(Block { assignments, expr })))
        }
    }
}

#[derive(Debug)]
pub(super) struct Package {
    pub(crate) pkg: Pkg,
    pub(crate) types: Context,
    pub(crate) symbols: Symbols,
    pub(crate) assignments: Vec<L<TLAssignment>>,
    pub(crate) expr: Option<L<Expr>>,
}
