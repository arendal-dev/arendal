mod twi;
mod typecheck;

use crate::ast::UnaryOp;
use crate::error::{Error, Loc, Result};
use crate::symbol::{Path, Symbol};
use crate::types::Type;
use crate::Integer;
use std::fmt;
use std::slice::Iter;
use std::sync::Arc;

use super::{Env, Value};

pub(super) fn run(env: &mut Env, path: &Path, input: &str) -> Result<Value> {
    let parsed = crate::ast::parser::parse(input)?;
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
struct Conditional {
    expr: Expression,
    then: Expression,
    otherwise: Expression,
}

impl Conditional {
    fn new(expr: Expression, then: Expression, otherwise: Expression) -> Result<Arc<Self>> {
        Error::merge(expr.check_boolean(), Self::same_types(&then, &otherwise))?;
        Ok(Arc::new(Self {
            expr,
            then,
            otherwise,
        }))
    }

    fn same_types(then: &Expression, otherwise: &Expression) -> Result<()> {
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
    Conditional(Arc<Conditional>),
    Assignment(Arc<Assignment>),
    Unary(Arc<Unary>),
    IntAdd(Arc<TwoInts>),
    IntSub(Arc<TwoInts>),
    IntMul(Arc<TwoInts>),
    IntDiv(Arc<TwoInts>),
    LogicalAnd(Arc<TwoBools>),
    LogicalOr(Arc<TwoBools>),
    Block(Expressions),
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
            Self::Block(exprs) => exprs.borrow_type(),
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

    fn ok(&self, expr: Expr) -> Result<Expression> {
        Ok(self.build(expr))
    }

    fn value(&self, value: Value) -> Expression {
        self.build(Expr::Value(value))
    }

    fn v_none(&self) -> Expression {
        self.value(Value::v_none(&self.loc))
    }

    fn val_integer(&self, value: Integer) -> Expression {
        self.value(Value::integer(&self.loc, value))
    }

    fn local(&self, symbol: Symbol, tipo: Type) -> Expression {
        self.build(Expr::Local(Arc::new(Local { symbol, tipo })))
    }

    fn conditional(
        &self,
        expr: Expression,
        then: Expression,
        otherwise: Expression,
    ) -> Result<Expression> {
        self.ok(Expr::Conditional(Conditional::new(expr, then, otherwise)?))
    }

    fn assignment(&self, symbol: Symbol, expr: Expression) -> Expression {
        self.build(Expr::Assignment(Arc::new(Assignment { symbol, expr })))
    }

    fn unary(&self, op: UnaryOp, expr: Expression) -> Expression {
        self.build(Expr::Unary(Arc::new(Unary { op, expr })))
    }

    fn int_add(&self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        self.ok(Expr::IntAdd(TwoInts::new(expr1, expr2)?))
    }

    fn int_sub(&self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        self.ok(Expr::IntSub(TwoInts::new(expr1, expr2)?))
    }

    fn int_mul(&self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        self.ok(Expr::IntMul(TwoInts::new(expr1, expr2)?))
    }

    fn int_div(&self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        self.ok(Expr::IntDiv(TwoInts::new(expr1, expr2)?))
    }

    fn log_and(&self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        self.ok(Expr::LogicalAnd(TwoBools::new(expr1, expr2)?))
    }

    fn log_or(&self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        self.ok(Expr::LogicalOr(TwoBools::new(expr1, expr2)?))
    }

    fn block(&self, exprs: Vec<Expression>) -> Result<Expression> {
        if exprs.is_empty() {
            Ok(self.v_none())
        } else {
            self.ok(Expr::Block(Expressions::new(exprs)))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDefinition {
    pub loc: Loc,
    pub tipo: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypeDefinitions {
    types: Vec<TypeDefinition>,
}

impl TypeDefinitions {
    pub fn new(types: Vec<TypeDefinition>) -> Self {
        Self { types }
    }

    pub fn iter(&self) -> Iter<'_, TypeDefinition> {
        self.types.iter()
    }
}

impl<'a> IntoIterator for &'a TypeDefinitions {
    type Item = &'a TypeDefinition;
    type IntoIter = Iter<'a, TypeDefinition>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    fn borrow_type(&self) -> &Type {
        if self.expressions.is_empty() {
            &Type::None
        } else {
            self.expressions.last().unwrap().borrow_type()
        }
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
    types: TypeDefinitions,
    expressions: Expressions,
}
