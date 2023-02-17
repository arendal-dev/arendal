use super::ValueResult;
use ast::{error::Errors, BinaryOp, Loc, Type, TypedExpression};
use num::Integer;

use super::{RuntimeError, TypedValue, Value};

pub(crate) fn eval<L: Loc + 'static>(expr: TypedExpression<L>) -> ValueResult {
    Eval::new(expr).eval()
}

struct Eval<L: Loc> {
    expr: TypedExpression<L>,
}

fn ok(value: Value, value_type: Type) -> ValueResult {
    Ok(TypedValue::new(value, value_type))
}

fn integer(value: Integer) -> ValueResult {
    ok(Value::Integer(value.clone()), Type::Integer)
}

impl<L: Loc + 'static> Eval<L> {
    fn new(expr: TypedExpression<L>) -> Self {
        Eval { expr }
    }

    fn eval_child(&self, expr: TypedExpression<L>) -> ValueResult {
        todo!()
    }

    fn loc(&self) -> L {
        self.expr.borrow_payload().loc.clone()
    }

    fn loc_type(&self) -> Type {
        self.expr.borrow_payload().loc_type.clone()
    }

    fn err(self) -> ValueResult {
        let mut errors: Errors = Default::default();
        errors.add(RuntimeError::new(self.loc()));
        Err(errors)
    }

    fn eval(self) -> ValueResult {
        match self.expr.borrow_expr() {
            ast::Expr::LitInteger(i) => integer(i.clone()),
            _ => self.err(),
        }
    }

    fn binary(self, op: BinaryOp, e1: TypedExpression<L>, e2: TypedExpression<L>) -> ValueResult {
        self.err()
    }
}
