use ast::{self, AST, problem::Output, symbol::FQPath};

use crate::{
    GlobalScope,
    resolved::{Binary, Expr, Expression, Resolved},
};

pub(super) fn resolve(path: FQPath, global: &GlobalScope, ast: &AST) -> Output<Resolved> {
    match &ast.expression {
        None => Output::ok(None),
        Some(e) => resolve_expression(&e).map(|e| Some(e)),
    }
    .map(|e| Resolved {
        path,
        expression: e,
    })
}

fn resolve_expression(expression: &ast::Expression) -> Output<Expression> {
    match &expression.expr {
        ast::Expr::LitInteger(num) => {
            Output::ok(Expr::LitInteger(num.clone()).wrap_from(expression))
        }
        ast::Expr::Binary(b) => resolve_expression(&b.expr1)
            .merge_to_tuple(resolve_expression(&b.expr2))
            .map(|(e1, e2)| {
                Expr::Binary(Binary {
                    op: b.op,
                    expr1: e1.into(),
                    expr2: e2.into(),
                })
                .wrap_from(expression)
            }),
        _ => panic!("TODO"),
    }
}
