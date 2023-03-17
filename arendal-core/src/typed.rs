use super::Integer;
use crate::ast::{BinaryOp, UnaryOp};
use crate::error::Loc;
use crate::identifier::Identifier;
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

    pub fn clone_loc(&self) -> Loc {
        self.inner.loc.clone()
    }

    pub fn borrow_type(&self) -> &Type {
        &self.inner.tipo
    }

    pub fn clone_type(&self) -> Type {
        self.inner.tipo.clone()
    }

    pub fn borrow_expr(&self) -> &TExpr {
        &self.inner.expr
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
    Val(Identifier),
    Assignment(Identifier, TypedExpr),
    Unary(UnaryOp, TypedExpr),
    Binary(BinaryOp, TypedExpr, TypedExpr),
}

pub struct TExprBuilder {
    loc: Loc,
}

impl TExprBuilder {
    pub const fn new(loc: Loc) -> Self {
        TExprBuilder { loc }
    }

    pub fn lit_integer(&self, value: Integer) -> TypedExpr {
        TypedExpr::new(self.loc.clone(), Type::integer(), TExpr::LitInteger(value))
    }

    pub fn lit_i64(&self, value: i64) -> TypedExpr {
        self.lit_integer(value.into())
    }

    pub fn val(&self, id: Identifier, tipo: Type) -> TypedExpr {
        TypedExpr::new(self.loc.clone(), tipo, TExpr::Val(id))
    }

    pub fn assignment(&self, id: Identifier, expr: TypedExpr) -> TypedExpr {
        TypedExpr::new(
            self.loc.clone(),
            expr.clone_type(),
            TExpr::Assignment(id, expr),
        )
    }

    pub fn unary(&self, tipo: Type, op: UnaryOp, expr: TypedExpr) -> TypedExpr {
        TypedExpr::new(self.loc.clone(), tipo, TExpr::Unary(op, expr))
    }

    pub fn binary(
        &self,
        tipo: Type,
        op: BinaryOp,
        expr1: TypedExpr,
        expr2: TypedExpr,
    ) -> TypedExpr {
        TypedExpr::new(self.loc.clone(), tipo, TExpr::Binary(op, expr1, expr2))
    }

    pub fn add(&self, tipo: Type, expr1: TypedExpr, expr2: TypedExpr) -> TypedExpr {
        self.binary(tipo, BinaryOp::Add, expr1, expr2)
    }

    pub fn add_i64(&self, value1: i64, value2: i64) -> TypedExpr {
        self.add(Type::integer(), self.lit_i64(value1), self.lit_i64(value2))
    }

    pub fn sub(&self, tipo: Type, expr1: TypedExpr, expr2: TypedExpr) -> TypedExpr {
        self.binary(tipo, BinaryOp::Sub, expr1, expr2)
    }

    pub fn sub_i64(&self, value1: i64, value2: i64) -> TypedExpr {
        self.sub(Type::integer(), self.lit_i64(value1), self.lit_i64(value2))
    }
}
