use super::{parse_expression, Expression};

use ast::helper::*;
use ast::Expr::*;

fn expr_eq(actual: &Expression, expected: &Expression) -> bool {
    let (e1, e2) = (actual.borrow_expr(), expected.borrow_expr());
    match e1 {
        LitInteger(v1) => {
            if let LitInteger(v2) = e2 {
                v1 == v2
            } else {
                false
            }
        }
        Unary(op1, ce1) => {
            if let Unary(op2, ce2) = e2 {
                op1 == op2 && expr_eq(ce1, ce2)
            } else {
                false
            }
        }
        Binary(op1, ce11, ce12) => {
            if let Binary(op2, ce21, ce22) = e2 {
                op1 == op2 && expr_eq(ce11, ce21) && expr_eq(ce12, ce22)
            } else {
                false
            }
        }
    }
}

fn check_expression(input: &str, expected: Expression) {
    let actual = parse_expression(input).unwrap();
    assert!(
        expr_eq(&actual, &expected),
        "\nActual: {:?}\nExpected: {:?}\n",
        actual,
        expected
    );
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
