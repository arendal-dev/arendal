use ast::error::{Errors, Result};
use ast::{Loc, Type, TypedExpression};

use super::{RuntimeError, TypedValue, Value};

pub(crate) fn eval<L: Loc + 'static>(expr: TypedExpression<L>) -> Result<TypedValue> {
    Eval::new(expr).eval()
}

struct Eval<L: Loc> {
    expr: TypedExpression<L>,
}

impl<L: Loc + 'static> Eval<L> {
    fn new(expr: TypedExpression<L>) -> Self {
        Eval { expr }
    }

    fn ok(&self, value: Value, value_type: Type) -> Result<TypedValue> {
        Ok(TypedValue::new(value, value_type))
    }

    fn loc(&self) -> L {
        self.expr.borrow_payload().loc.clone()
    }

    fn loc_type(&self) -> Type {
        self.expr.borrow_payload().loc_type.clone()
    }

    fn err(&self) -> Result<TypedValue> {
        let mut errors: Errors = Default::default();
        errors.add(RuntimeError { loc: self.loc() });
        Err(errors)
    }

    fn eval(self) -> Result<TypedValue> {
        match self.expr.borrow_expr() {
            ast::Expr::LitInteger(i) => self.ok(Value::Integer(i.clone()), self.loc_type()),
            _ => self.err(),
        }
    }
}
