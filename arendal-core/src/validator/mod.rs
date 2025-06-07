use std::marker::PhantomData;

use ast::{
    self, Statement,
    position::Position,
    problem::{Problems, Result},
    symbol::{FQSym, FQType},
};

use crate::itr::{self, EMPTY, Empty, Payload};

use crate::types::TypeExpr;

pub(super) fn validate(statements: Vec<Statement>) -> Result<ITR> {
    Validator::default().validate(statements)
}

#[derive(Debug, Eq)]
pub struct Valid {
    nothing: PhantomData<usize>,
}

impl Valid {
    fn new() -> Self {
        Valid {
            nothing: PhantomData,
        }
    }
}

impl PartialEq for Valid {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Payload for Valid {}

pub(crate) type Expression = itr::Expression<Option<TypeExpr>, Valid, FQSym, FQType>;
pub(crate) type Expr = itr::Expr<Option<TypeExpr>, Valid, FQSym, FQType>;
pub(crate) type ITR = itr::ITR<Option<TypeExpr>, Valid, FQSym, FQType>;
pub(crate) type Binary = itr::Binary<Option<TypeExpr>, Valid, FQSym, FQType>;

fn valid(expr: Expr, position: &Position) -> Expression {
    expr.to_expression(position.clone(), None, Valid::new())
}

#[derive(Default)]
struct Validator {
    problems: Problems,
}

impl Validator {
    fn validate(mut self, statements: Vec<Statement>) -> Result<ITR> {
        let option = if statements.is_empty() {
            None
        } else if statements.len() > 1 {
            panic!("TODO");
        } else {
            match &statements[0] {
                Statement::Expression(expression) => self.validate_expression(&expression),
            }
        };
        self.problems.to_result(ITR { expression: option })
    }

    fn validate_expression(&mut self, expression: &ast::Expression) -> Option<Expression> {
        match expression.expr() {
            ast::Expr::LitInteger(num) => ok_expr(Expr::LitInteger(num.clone()), expression),
            ast::Expr::Binary(b) => {
                let option1 = self.validate_expression(&b.expr1);
                let option2 = self.validate_expression(&b.expr2);
                // We extract from the option later to collect as many problems as possible.
                let expr1 = option1?;
                let expr2 = option2?;
                ok_expr(
                    Expr::Binary(Binary {
                        op: b.op,
                        expr1,
                        expr2,
                    }),
                    expression,
                )
            }
            _ => panic!("TODO"),
        }
    }
}

fn ok_expr(expr: Expr, expression: &ast::Expression) -> Option<Expression> {
    Some(valid(expr, expression.position()))
}
