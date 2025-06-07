use ast::{
    problem::{Problems, Result},
    symbol::{FQSym, FQType},
};

use crate::itr;

use crate::types::Type;
use crate::validator;

pub(super) fn typecheck(tree: validator::ITR) -> Result<ITR> {
    TypeChecker::default().typecheck(tree)
}

#[derive(Debug, Eq)]
pub struct Checked {
    _nothing: (),
}

impl Checked {
    fn new() -> Self {
        Checked { _nothing: () }
    }
}

impl PartialEq for Checked {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl itr::Payload for Checked {}

pub(crate) type Expression = itr::Expression<Type, Checked, FQSym, FQType>;
pub(crate) type Expr = itr::Expr<Type, Checked, FQSym, FQType>;
pub(crate) type ITR = itr::ITR<Type, Checked, FQSym, FQType>;
pub(crate) type Binary = itr::Binary<Type, Checked, FQSym, FQType>;

#[derive(Default)]
struct TypeChecker {
    problems: Problems,
}

impl TypeChecker {
    fn typecheck(mut self, input: validator::ITR) -> Result<ITR> {
        panic!("TODO")
    }

    fn typecheck_expression(&mut self, expression: &validator::Expression) -> Option<Expression> {
        match expression.expr() {
            validator::Expr::LitInteger(num) => Some(new_e(
                expression,
                Expr::LitInteger(num.clone()),
                Type::Integer,
            )),
            validator::Expr::Binary(b) => {
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

    fn typecheck_binary(
        &mut self,
        expression: &validator::Expression,
        b: &validator::Binary,
    ) -> Expression {
        panic!("TODO")
    }
}

fn new_e(from: &validator::Expression, expr: Expr, t: Type) -> Expression {
    expr.to_expression(from.position().clone(), t, Checked::new())
}

pub(crate) enum TypeError {
    TypeError,
}
