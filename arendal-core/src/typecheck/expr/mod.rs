use crate::ast::{BinaryOp, Expr, Expression};
use crate::error::{Errors, Loc, Result};
use crate::typed::TypedExpr;
use crate::types::Type;

use super::TypeError;

pub(crate) fn check(input: Expression) -> Result<TypedExpr> {
    Checker::new(input).check()
}

struct Checker {
    input: Expression,
}

impl Checker {
    fn new(input: Expression) -> Self {
        Checker { input }
    }

    fn loc(&self) -> Loc {
        self.input.borrow_loc().clone()
    }

    // Creates and returns an error
    fn error(self, kind: TypeError) -> Result<TypedExpr> {
        let mut errors: Errors = Default::default();
        errors.add(self.loc(), kind);
        Err(errors)
    }

    fn check(self) -> Result<TypedExpr> {
        match self.input.borrow_expr() {
            Expr::LitInteger(value) => Ok(TypedExpr::lit_integer(self.loc(), value.clone())),
            Expr::Binary(op, e1, e2) => {
                let c1 = Self::new(e1.clone()).check();
                let c2 = Self::new(e2.clone()).check();
                if c1.is_err() || c2.is_err() {
                    let mut errors: Errors = Default::default();
                    c1.map_err(|e| errors.append(e));
                    c2.map_err(|e| errors.append(e));
                    Err(errors)
                } else {
                    let e1 = c1.unwrap();
                    let e2 = c2.unwrap();
                    match op {
                        BinaryOp::Add => self.check_add(e1, e2),
                        BinaryOp::Sub => self.check_sub(e1, e2),
                        BinaryOp::Mul => self.check_mul(e1, e2),
                        BinaryOp::Div => self.check_div(e1, e2),
                        _ => self.error(TypeError::InvalidType),
                    }
                }
            }
            _ => self.error(TypeError::InvalidType),
        }
    }

    fn ok_binary(
        &self,
        tipo: Type,
        op: BinaryOp,
        e1: TypedExpr,
        e2: TypedExpr,
    ) -> Result<TypedExpr> {
        Ok(TypedExpr::binary(self.loc(), tipo, op, e1, e2))
    }

    fn check_add(self, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::integer(), BinaryOp::Add, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }

    fn check_sub(self, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::integer(), BinaryOp::Sub, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }

    fn check_mul(self, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::integer(), BinaryOp::Mul, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }

    fn check_div(self, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::integer(), BinaryOp::Div, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }
}

#[cfg(test)]
mod tests;
