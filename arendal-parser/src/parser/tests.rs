use super::parse_expression;

use ast::bare;
use ast::bare::Expression;

fn check_expression(input: &str, expr: Expression) {
    assert_eq!(parse_expression(input).unwrap().to_bare(), expr);
}

fn int_literal(value: i64) -> Expression {
    bare::lit_integer(value.into())
}

#[test]
fn int_literal_expr() {
    check_expression("1234", int_literal(1234));
}
