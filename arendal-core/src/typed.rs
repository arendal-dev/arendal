use super::Integer;
use crate::ast::{BinaryOp, UnaryOp};
use crate::error::Loc;
use crate::symbol::Symbol;
use crate::types::Type;
use crate::value::Value;
use std::fmt;
use std::rc::Rc;
use std::slice::Iter;

#[derive(Debug)]
struct Inner {
    loc: Loc,
    tipo: Type,
    expr: Expr,
}

#[derive(Clone)]
pub struct Expression {
    inner: Rc<Inner>,
}

impl Expression {
    fn new(loc: Loc, tipo: Type, expr: Expr) -> Self {
        Expression {
            inner: Rc::new(Inner { loc, tipo, expr }),
        }
    }

    pub fn borrow_loc(&self) -> &Loc {
        &self.inner.loc
    }

    pub fn clone_loc(&self) -> Loc {
        self.inner.loc.clone()
    }

    pub fn borrow_type(&self) -> &Type {
        &self.inner.tipo
    }

    pub fn clone_type(&self) -> Type {
        self.inner.tipo.clone()
    }

    pub fn borrow_expr(&self) -> &Expr {
        &self.inner.expr
    }

    pub fn is_integer(&self) -> bool {
        self.inner.tipo == Type::Integer
    }

    pub fn is_boolean(&self) -> bool {
        self.inner.tipo == Type::Boolean
    }

    pub fn is_boolean_true(&self) -> bool {
        self.inner.tipo == Type::True
    }

    pub fn is_boolean_false(&self) -> bool {
        self.inner.tipo == Type::False
    }

    pub fn is_none(&self) -> bool {
        self.inner.tipo == Type::None
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} : {:?}", self.borrow_expr(), self.borrow_type())
    }
}

#[derive(Debug)]
pub enum Expr {
    Value(Value),
    LocalSymbol(Symbol),
    Assignment(Symbol, Expression),
    Unary(UnaryOp, Expression),
    Binary(BinaryOp, Expression, Expression),
}

pub struct ExprBuilder {
    loc: Loc,
}

impl ExprBuilder {
    pub const fn new(loc: Loc) -> Self {
        ExprBuilder { loc }
    }

    pub fn value(&self, value: Value) -> Expression {
        Expression::new(self.loc.clone(), value.clone_type(), Expr::Value(value))
    }

    pub fn val_integer(&self, value: Integer) -> Expression {
        self.value(Value::Integer(value))
    }

    pub fn val_i64(&self, value: i64) -> Expression {
        self.val_integer(value.into())
    }

    pub fn val(&self, id: Symbol, tipo: Type) -> Expression {
        Expression::new(self.loc.clone(), tipo, Expr::LocalSymbol(id))
    }

    pub fn assignment(&self, id: Symbol, expr: Expression) -> Expression {
        Expression::new(
            self.loc.clone(),
            expr.clone_type(),
            Expr::Assignment(id, expr),
        )
    }

    pub fn unary(&self, tipo: Type, op: UnaryOp, expr: Expression) -> Expression {
        Expression::new(self.loc.clone(), tipo, Expr::Unary(op, expr))
    }

    pub fn binary(
        &self,
        tipo: Type,
        op: BinaryOp,
        expr1: Expression,
        expr2: Expression,
    ) -> Expression {
        Expression::new(self.loc.clone(), tipo, Expr::Binary(op, expr1, expr2))
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
    pub(crate) expressions: Expressions,
}
