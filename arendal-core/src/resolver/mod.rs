use ast::{
    self, Statement,
    problem::{Result, merge, ok},
};

use crate::{
    GlobalScope,
    resolved::{Binary, Expr, Expression, Resolved},
};

pub(super) fn resolve(global: &GlobalScope, statements: &Vec<Statement>) -> Result<Resolved> {
    if statements.is_empty() {
        ok(None)
    } else if statements.len() > 1 {
        panic!("TODO");
    } else {
        match &statements[0] {
            Statement::Expression(expression) => {
                resolve_expression(&expression)?.and_then(|e| ok(Some(e)))
            }
        }
    }?
    .and_then(|e| ok(Resolved { expression: e }))
}

fn resolve_expression(expression: &ast::Expression) -> Result<Expression> {
    match &expression.expr {
        ast::Expr::LitInteger(num) => ok(Expr::LitInteger(num.clone()).wrap_from(expression)),
        ast::Expr::Binary(b) => {
            merge(resolve_expression(&b.expr1), resolve_expression(&b.expr2))?.and_then(
                |(e1,e2)|
                // We extract from the option later to collect as many problems as possible.
                ok(
                    Expr::Binary(Binary {
                        op: b.op,
                        expr1: e1.into(),
                        expr2: e2.into(),
                    })
                    .wrap_from(expression),
                ),
            )
        }
        _ => panic!("TODO"),
    }
}
