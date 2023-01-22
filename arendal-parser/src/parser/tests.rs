use super::parse_expression;

use arendal_ast::bare;
use arendal_ast::bare::Expression;
use arendal_ast::ToBigInt;

fn check_expression(input: &str, expr: Expression) {
    assert_eq!(parse_expression(input).unwrap().to_bare(), expr);
}

fn int_literal(value: usize) -> Expression {
    bare::int_literal(value.to_bigint().unwrap())
}

#[test]
fn int_literal_expr() {
    check_expression("1234", int_literal(1234));
}
