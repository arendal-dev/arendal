use ast::{
    problem::{Problems, Result},
    symbol::{FQSym, FQType},
};

use crate::{
    resolved,
    typechecked::{Expr, Expression, TypeChecked},
    types::{Type, Value},
};

use crate::resolver;
use crate::types::TypeExpr;

pub(super) fn typecheck(tree: resolved::Resolved) -> Result<TypeChecked> {
    TypeChecker::default().typecheck(tree)
}

#[derive(Default)]
struct TypeChecker {
    problems: Problems,
}

impl TypeChecker {
    fn typecheck(mut self, input: resolved::Resolved) -> Result<TypeChecked> {
        panic!("TODO")
    }

    fn typecheck_expression(&mut self, expression: &resolved::Expression) -> Option<Expression> {
        match &expression.expr {
            resolved::Expr::LitInteger(num) => Some(new_e(
                expression,
                Expr::Value(Value::Integer(num.clone())),
                TypeExpr::Type(Type::Integer),
            )),
            resolved::Expr::Binary(b) => {
                let option1 = self.typecheck_expression(&b.expr1);
                let option2 = self.typecheck_expression(&b.expr2);
                // We extract from the option later to collect as many problems as possible.
                let expr1 = option1?;
                let expr2 = option2?;
                panic!("TODO")
            }
            _ => panic!("TODO"),
        }
    }
}

fn new_e(from: &resolved::Expression, expr: Expr, t: TypeExpr) -> Expression {
    expr.wrap(from.position.clone(), t)
}

pub(crate) enum TypeError {
    TypeError,
}
