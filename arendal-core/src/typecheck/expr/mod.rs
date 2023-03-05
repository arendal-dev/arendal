use crate::ast::{BinaryOp, Expr, Expression};
use crate::error::{Errors, Loc, Result};
use crate::scope::Scope;
use crate::typed::TypedExpr;
use crate::types::Type;

use super::{Checked, CheckedExpr, TypeError};

pub(crate) fn check(scope: Scope, input: Expression) -> Result<CheckedExpr> {
    Checker::new(scope, input).check()
}

struct Checker {
    scope: Scope,
    input: Expression,
    errors: Errors,
}

impl Checker {
    fn new(scope: Scope, input: Expression) -> Self {
        Checker {
            scope,
            input,
            errors: Default::default(),
        }
    }

    fn loc(&self) -> Loc {
        self.input.clone_loc()
    }

    // Adds and returns an error
    fn error(mut self, kind: TypeError) -> Result<CheckedExpr> {
        self.errors.add(self.loc(), kind);
        Err(self.errors)
    }

    fn ok(self, e: TypedExpr) -> Result<CheckedExpr> {
        self.errors.to_result(Checked::new(self.scope, e))
    }

    fn check_child(&mut self, e: Expression) -> Option<TypedExpr> {
        match self
            .errors
            .append_result(Self::new(self.scope.clone(), e).check())
        {
            Some(checked) => {
                self.scope = checked.scope;
                Some(checked.it)
            }
            None => None,
        }
    }

    fn check(mut self) -> Result<CheckedExpr> {
        match self.input.clone_expr() {
            Expr::LitInteger(value) => {
                let loc = self.loc();
                self.ok(TypedExpr::lit_integer(loc, value.clone()))
            }
            Expr::Binary(op, e1, e2) => {
                let t1 = self.check_child(e1.clone());
                let t2 = self.check_child(e2.clone());
                if t1.is_some() && t2.is_some() {
                    let e1 = t1.unwrap();
                    let e2 = t2.unwrap();
                    match op {
                        BinaryOp::Add => self.check_add(e1, e2),
                        BinaryOp::Sub => self.check_sub(e1, e2),
                        BinaryOp::Mul => self.check_mul(e1, e2),
                        BinaryOp::Div => self.check_div(e1, e2),
                        _ => self.error(TypeError::InvalidType),
                    }
                } else {
                    Err(self.errors)
                }
            }
            _ => self.error(TypeError::InvalidType),
        }
    }

    fn ok_binary(
        self,
        tipo: Type,
        op: BinaryOp,
        e1: TypedExpr,
        e2: TypedExpr,
    ) -> Result<CheckedExpr> {
        let loc = self.loc();
        self.ok(TypedExpr::binary(loc, tipo, op, e1, e2))
    }

    fn check_add(self, e1: TypedExpr, e2: TypedExpr) -> Result<CheckedExpr> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::integer(), BinaryOp::Add, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }

    fn check_sub(self, e1: TypedExpr, e2: TypedExpr) -> Result<CheckedExpr> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::integer(), BinaryOp::Sub, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }

    fn check_mul(self, e1: TypedExpr, e2: TypedExpr) -> Result<CheckedExpr> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::integer(), BinaryOp::Mul, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }

    fn check_div(self, e1: TypedExpr, e2: TypedExpr) -> Result<CheckedExpr> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::integer(), BinaryOp::Div, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }
}

#[cfg(test)]
mod tests;
