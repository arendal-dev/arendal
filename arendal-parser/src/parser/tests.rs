use super::parse_expression;

use ast::bare::*;
use ast::BinaryOp;

fn check_expression(input: &str, expr: Expression) {
    assert_eq!(parse_expression(input).unwrap().to_bare(), expr);
}

#[test]
fn int_literal_expr() {
    check_expression("1234", lit_i64(1234));
}

#[test]
fn sum1() {
    check_expression("1+2", add_i64(1, 2));
}

#[test]
fn sum2() {
    check_expression("1 + 2", add_i64(1, 2));
}

#[test]
fn sum3() {
    check_expression("\t1 + 2", add_i64(1, 2));
}

#[test]
fn sum4() {
    check_expression("1 + 2 + 3", add(add_i64(1, 2), lit_i64(3)));
}
