use super::parse_expression;

use ast::bare::Expression;
use ast::{bare, BinaryOp};

fn check_expression(input: &str, expr: Expression) {
    assert_eq!(parse_expression(input).unwrap().to_bare(), expr);
}

fn int_literal(value: i64) -> Expression {
    bare::lit_integer(value.into())
}

fn add(left: Expression, right: Expression) -> Expression {
    bare::binary(BinaryOp::Add, left, right)
}

fn add_int(left: i64, right: i64) -> Expression {
    add(int_literal(left), int_literal(right))
}

#[test]
fn int_literal_expr() {
    check_expression("1234", int_literal(1234));
}

#[test]
fn sum1() {
    check_expression("1+2", add_int(1, 2));
}

#[test]
fn sum2() {
    check_expression("1 + 2", add_int(1, 2));
}

#[test]
fn sum3() {
    check_expression("\t1 + 2", add_int(1, 2));
}

#[test]
fn sum4() {
    check_expression("1 + 2 + 3", add(add_int(1, 2), int_literal(3)));
}
