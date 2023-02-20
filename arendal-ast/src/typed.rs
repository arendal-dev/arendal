use super::{BinaryOp, Integer, Loc, Type, UnaryOp};
use std::rc::Rc;

#[derive(Debug)]
struct Inner {
    loc: Loc,
    expr_type: Type,
    expr: TExpr,
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    inner: Rc<Inner>,
}

impl TypedExpr {
    fn new(loc: Loc, expr_type: Type, expr: TExpr) -> Self {
        TypedExpr {
            inner: Rc::new(Inner {
                loc,
                expr_type,
                expr,
            }),
        }
    }

    pub fn borrow_loc(&self) -> &Loc {
        &self.inner.loc
    }

    pub fn borrow_type(&self) -> &Type {
        &self.inner.expr_type
    }

    pub fn borrow_expr(&self) -> &TExpr {
        &self.inner.expr
    }

    pub fn lit_integer(loc: Loc, value: Integer) -> Self {
        Self::new(loc, Type::Integer, TExpr::LitInteger(value))
    }

    pub fn unary(loc: Loc, expr_type: Type, op: UnaryOp, expr: TypedExpr) -> Self {
        Self::new(loc, expr_type, TExpr::Unary(op, expr))
    }

    pub fn binary(
        loc: Loc,
        expr_type: Type,
        op: BinaryOp,
        expr1: TypedExpr,
        expr2: TypedExpr,
    ) -> Self {
        Self::new(loc, expr_type, TExpr::Binary(op, expr1, expr2))
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

    pub fn unary(expr_type: Type, op: UnaryOp, expr: TypedExpr) -> TypedExpr {
        TypedExpr::unary(Loc::none(), expr_type, op, expr)
    }

    pub fn binary(expr_type: Type, op: BinaryOp, expr1: TypedExpr, expr2: TypedExpr) -> TypedExpr {
        TypedExpr::binary(Loc::none(), expr_type, op, expr1, expr2)
    }

    pub fn add(expr_type: Type, expr1: TypedExpr, expr2: TypedExpr) -> TypedExpr {
        binary(expr_type, BinaryOp::Add, expr1, expr2)
    }

    pub fn add_i64(value1: i64, value2: i64) -> TypedExpr {
        add(Type::Integer, lit_i64(value1), lit_i64(value2))
    }
}
