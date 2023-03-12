use core::ast::BinaryOp;
use core::error::{Errors, Loc};
use core::typed::TExpr::*;
use core::typed::TypedExpr;
use core::types::Type;
use core::Integer;

use crate::value::Value;
use crate::{RuntimeError, ValueResult};

pub(crate) fn eval(expr: TypedExpr) -> ValueResult {
    Eval::new(expr).eval()
}

struct Eval {
    expr: TypedExpr,
}

fn integer(value: Integer) -> ValueResult {
    Ok(Value::Int(value))
}

impl Eval {
    fn new(expr: TypedExpr) -> Self {
        Eval { expr }
    }

    fn eval_child(&self, expr: TypedExpr) -> ValueResult {
        Self::new(expr).eval()
    }

    fn loc(&self) -> Loc {
        self.expr.borrow_loc().clone()
    }

    fn loc_type(&self) -> Type {
        self.expr.borrow_type().clone()
    }

    fn err(&self) -> ValueResult {
        Err(Errors::new(self.loc(), RuntimeError {}))
    }

    fn eval(self) -> ValueResult {
        match self.expr.borrow_expr() {
            LitInteger(i) => integer(i.clone()),
            Binary(op, e1, e2) => self.binary(*op, e1.clone(), e2.clone()),

            _ => self.err(),
        }
    }

    fn binary(&self, op: BinaryOp, e1: TypedExpr, e2: TypedExpr) -> ValueResult {
        let v1 = self.eval_child(e1)?;
        match op {
            BinaryOp::Add => self.add(v1, e2),
            BinaryOp::Sub => self.sub(v1, e2),
            BinaryOp::Mul => self.mul(v1, e2),
            BinaryOp::Div => self.div(v1, e2),
            _ => self.err(),
        }
    }

    fn add(&self, v1: Value, e2: TypedExpr) -> ValueResult {
        let v2 = self.eval_child(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() + v2.as_integer().unwrap())
    }

    fn sub(&self, v1: Value, e2: TypedExpr) -> ValueResult {
        let v2 = self.eval_child(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() - v2.as_integer().unwrap())
    }

    fn mul(&self, v1: Value, e2: TypedExpr) -> ValueResult {
        let v2 = self.eval_child(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() * v2.as_integer().unwrap())
    }

    fn div(&self, v1: Value, e2: TypedExpr) -> ValueResult {
        let v2 = self.eval_child(e2)?;
        // We only have integers for now
        let i2 = v2.as_integer().unwrap();
        if i2.is_zero() {
            self.err()
        } else {
            integer(v1.as_integer().unwrap() / i2)
        }
    }
}

#[cfg(test)]
mod tests;
