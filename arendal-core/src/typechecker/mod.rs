use ast::{
    problem::Output,
    symbol::{FQSym, FQType},
};

use crate::{
    resolved,
    typechecked::{Expr, Expression, TypeChecked},
    types::{Type, Value},
};

use crate::types::TypeExpr;

pub(super) fn typecheck(tree: resolved::Resolved) -> Output<TypeChecked> {
    let mut type_checked = TypeChecked {
        path: tree.path,
        expression: None,
    };
    match tree.expression {
        Some(e) => typecheck_expression(&e).map(|te| {
            type_checked.expression = Some(te);
            type_checked
        }),
        _ => Output::ok(type_checked),
    }
}

fn typecheck_expression(expression: &resolved::Expression) -> Output<Expression> {
    match &expression.expr {
        resolved::Expr::LitInteger(num) => Output::ok(
            Expr::Value(Value::Integer(num.clone()))
                .wrap_from(expression, TypeExpr::Type(Type::Integer)),
        ),
        resolved::Expr::Binary(b) => {
            let _option1 = typecheck_expression(&b.expr1);
            let _option2 = typecheck_expression(&b.expr2);
            // We extract from the option later to collect as many problems as possible.
            // let expr1 = option1?;
            // let expr2 = option2?;
            panic!("TODO")
        }
        _ => panic!("TODO"),
    }
}

pub(crate) enum TypeError {
    TypeError,
}
