use super::Integer;
use crate::ast::{BinaryOp, UnaryOp};
use crate::error::Loc;
use crate::types::Type;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
struct Inner {
    loc: Loc,
    tipo: Type,
    expr: TExpr,
}

#[derive(Clone)]
pub struct TypedExpr {
    inner: Rc<Inner>,
}

impl TypedExpr {
    fn new(loc: Loc, tipo: Type, expr: TExpr) -> Self {
        TypedExpr {
            inner: Rc::new(Inner { loc, tipo, expr }),
        }
    }

    pub fn borrow_loc(&self) -> &Loc {
        &self.inner.loc
    }

    pub fn borrow_type(&self) -> &Type {
        &self.inner.tipo
    }

    pub fn borrow_expr(&self) -> &TExpr {
        &self.inner.expr
    }

    pub fn lit_integer(loc: Loc, value: Integer) -> Self {
        Self::new(loc, Type::integer(), TExpr::LitInteger(value))
    }

    pub fn unary(loc: Loc, tipo: Type, op: UnaryOp, expr: TypedExpr) -> Self {
        Self::new(loc, tipo, TExpr::Unary(op, expr))
    }

    pub fn binary(loc: Loc, tipo: Type, op: BinaryOp, expr1: TypedExpr, expr2: TypedExpr) -> Self {
        Self::new(loc, tipo, TExpr::Binary(op, expr1, expr2))
    }

    pub fn is_integer(&self) -> bool {
        self.inner.tipo.is_integer()
    }

    pub fn is_boolean(&self) -> bool {
        self.inner.tipo.is_boolean()
    }

    pub fn is_boolean_true(&self) -> bool {
        self.inner.tipo.is_boolean_true()
    }

    pub fn is_boolean_false(&self) -> bool {
        self.inner.tipo.is_boolean_false()
    }

    pub fn is_none(&self) -> bool {
        self.inner.tipo.is_none()
    }

    pub fn is_some(&self) -> bool {
        self.inner.tipo.is_some()
    }

    pub fn is_option(&self) -> bool {
        self.inner.tipo.is_option()
    }
}

impl fmt::Debug for TypedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} : {:?}", self.borrow_expr(), self.borrow_type())
    }
}

#[derive(Debug)]
pub enum TExpr {
    LitInteger(Integer),
    Unary(UnaryOp, TypedExpr),
    Binary(BinaryOp, TypedExpr, TypedExpr),
}

pub mod helper {
    use super::{BinaryOp, Integer, Loc, Type, TypedExpr, UnaryOp};

    pub fn lit_integer(value: Integer) -> TypedExpr {
        TypedExpr::lit_integer(Loc::none(), value)
    }

    pub fn lit_i64(value: i64) -> TypedExpr {
        lit_integer(value.into())
    }

    pub fn unary(tipo: Type, op: UnaryOp, expr: TypedExpr) -> TypedExpr {
        TypedExpr::unary(Loc::none(), tipo, op, expr)
    }

    pub fn binary(tipo: Type, op: BinaryOp, expr1: TypedExpr, expr2: TypedExpr) -> TypedExpr {
        TypedExpr::binary(Loc::none(), tipo, op, expr1, expr2)
    }

    pub fn add(tipo: Type, expr1: TypedExpr, expr2: TypedExpr) -> TypedExpr {
        binary(tipo, BinaryOp::Add, expr1, expr2)
    }

    pub fn add_i64(value1: i64, value2: i64) -> TypedExpr {
        add(Type::integer(), lit_i64(value1), lit_i64(value2))
    }

    pub fn sub(tipo: Type, expr1: TypedExpr, expr2: TypedExpr) -> TypedExpr {
        binary(tipo, BinaryOp::Sub, expr1, expr2)
    }

    pub fn sub_i64(value1: i64, value2: i64) -> TypedExpr {
        sub(Type::integer(), lit_i64(value1), lit_i64(value2))
    }
}
