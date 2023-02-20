use super::TExpr::*;
use super::{BinaryOp, Errors, Loc, Type, TypedExpr, ValueResult};
use num::Integer;

use super::{RuntimeError, Value};

pub(crate) fn eval(expr: TypedExpr) -> ValueResult {
    Eval::new(expr).eval()
}

struct Eval {
    expr: TypedExpr,
}

fn integer(value: Integer) -> ValueResult {
    Ok(Value::Integer(value))
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
        let mut errors: Errors = Default::default();
        errors.add(RuntimeError::new(self.loc()));
        Err(errors)
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
            _ => self.err(),
        }
    }

    fn add(&self, v1: Value, e2: TypedExpr) -> ValueResult {
        let v2 = self.eval_child(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() + v2.as_integer().unwrap())
    }
}

#[cfg(test)]
mod tests;
