use std::marker::PhantomData;

use ast::{
    Payload,
    position::Position,
    problem::{Problems, Result},
    stmt::{self, Statement},
};

pub(super) fn validate(statements: Vec<Statement>) -> Result<AST> {
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

type Expression = ast::Expression<Valid>;
type Expr = ast::Expr<Valid>;
type AST = ast::AST<Valid>;
type Binary = ast::Binary<Valid>;

trait Lift<T> {
    fn lift(self, position: &Position) -> T;
}

impl Lift<Expression> for Expr {
    fn lift(self, position: &Position) -> Expression {
        self.to_expression(position.clone(), Valid::new())
    }
}

#[derive(Default)]
struct Validator {
    problems: Problems,
}

impl Validator {
    fn validate(mut self, statements: Vec<Statement>) -> Result<AST> {
        let option = if statements.is_empty() {
            None
        } else if statements.len() > 1 {
            panic!("TODO");
        } else {
            match &statements[0] {
                Statement::Expression(expression) => self.validate_expression(&expression),
            }
        };
        self.problems.to_result(AST { expression: option })
    }

    fn validate_expression(&mut self, expression: &stmt::Expression) -> Option<Expression> {
        match expression.expr() {
            stmt::Expr::LitInteger(num) => ok_expr(Expr::LitInteger(num.clone()), expression),
            stmt::Expr::Binary(b) => {
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

fn ok_expr(expr: Expr, expression: &stmt::Expression) -> Option<Expression> {
    Some(expr.lift(expression.position()))
}
