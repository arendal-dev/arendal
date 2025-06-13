use ast::{
    self, Statement,
    problem::{Problems, Result},
};

use crate::resolved::{Binary, Expr, Expression, Resolved};

use crate::types::TypeExpr;

pub(super) fn resolve(statements: Vec<Statement>) -> Result<Resolved> {
    Validator::default().validate(statements)
}

#[derive(Default)]
struct Validator {
    problems: Problems,
}

impl Validator {
    fn validate(mut self, statements: Vec<Statement>) -> Result<Resolved> {
        let option = if statements.is_empty() {
            None
        } else if statements.len() > 1 {
            panic!("TODO");
        } else {
            match &statements[0] {
                Statement::Expression(expression) => self.validate_expression(&expression),
            }
        };
        self.problems.to_result(Resolved { expression: option })
    }

    fn validate_expression(&mut self, expression: &ast::Expression) -> Option<Expression> {
        match &expression.expr {
            ast::Expr::LitInteger(num) => Some(Expr::LitInteger(num.clone()).wrap_from(expression)),
            ast::Expr::Binary(b) => {
                let option1 = self.validate_expression(&b.expr1);
                let option2 = self.validate_expression(&b.expr2);
                // We extract from the option later to collect as many problems as possible.
                let expr1 = option1?;
                let expr2 = option2?;
                Some(
                    Expr::Binary(Binary {
                        op: b.op,
                        expr1: expr1.into(),
                        expr2: expr2.into(),
                    })
                    .wrap_from(expression),
                )
            }
            _ => panic!("TODO"),
        }
    }
}
