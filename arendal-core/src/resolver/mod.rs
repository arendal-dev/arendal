use ast::{
    self, AST,
    problem::{Result, merge, ok},
};

use crate::{
    GlobalScope,
    resolved::{Binary, Expr, Expression, Resolved},
};

pub(super) fn resolve(global: &GlobalScope, ast: &AST) -> Result<Resolved> {
    match &ast.expression {
        None => ok(None),
        Some(e) => resolve_expression(&e)?.and_then(|e| ok(Some(e))),
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
