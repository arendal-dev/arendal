use ast::{
    position::EqNoPosition,
    stmt::{Expr, Expression, Statement},
};

fn check_statements(input: &str, expected: Vec<Statement>) {
    let (actual, _) = super::parse(input).unwrap();
    actual.assert_eq_nopos(&expected);
}

fn check_statement(input: &str, expected: Statement) {
    check_statements(input, vec![expected]);
}

fn check_expression(input: &str, expected: Expression) {
    check_statement(input, expected.to_statement());
}

fn e(expr: Expr) -> Expression {
    expr.to_expression(&ast::position::Position::NoPosition)
}

fn e_i64(value: i64) -> Expression {
    e(Expr::LitInteger(value.into()))
}

#[test]
fn int_literal_expr() {
    check_expression("1234", e_i64(1234));
    check_expression("  1234 ", e_i64(1234));
    check_expression("\t \n 1234  \n\t", e_i64(1234));
}
