use super::ValueResult;
use ast::{error::Errors, BinaryOp, Expr, Loc, Type, TypedExpression};
use num::Integer;

use super::{RuntimeError, Value};

pub(crate) fn eval<L: Loc + 'static>(expr: TypedExpression<L>) -> ValueResult {
    Eval::new(expr).eval()
}

struct Eval<L: Loc> {
    expr: TypedExpression<L>,
}

fn integer(value: Integer) -> ValueResult {
    Ok(Value::Integer(value))
}

impl<L: Loc + 'static> Eval<L> {
    fn new(expr: TypedExpression<L>) -> Self {
        Eval { expr }
    }

    fn eval_child(&self, expr: TypedExpression<L>) -> ValueResult {
        Self::new(expr).eval()
    }

    fn loc(&self) -> L {
        self.expr.borrow_payload().loc.clone()
    }

    fn loc_type(&self) -> Type {
        self.expr.borrow_payload().loc_type.clone()
    }

    fn err(&self) -> ValueResult {
        let mut errors: Errors = Default::default();
        errors.add(RuntimeError::new(self.loc()));
        Err(errors)
    }

    fn eval(self) -> ValueResult {
        match self.expr.borrow_expr() {
            Expr::LitInteger(i) => integer(i.clone()),
            Expr::Binary(op, e1, e2) => self.binary(*op, e1.clone(), e2.clone()),

            _ => self.err(),
        }
    }

    fn binary(&self, op: BinaryOp, e1: TypedExpression<L>, e2: TypedExpression<L>) -> ValueResult {
        let v1 = self.eval_child(e1)?;
        match op {
            BinaryOp::Add => self.add(v1, e2),
            _ => self.err(),
        }
    }

    fn add(&self, v1: Value, e2: TypedExpression<L>) -> ValueResult {
        let v2 = self.eval_child(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() + v2.as_integer().unwrap())
    }
}
