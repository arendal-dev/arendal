use core::ast::Expr::*;
use core::ast::{ExprBuilder, Expression};
use core::identifier::Identifier;

use super::parse_expression;

const B: ExprBuilder = ExprBuilder::none();

fn expr_eq(actual: &Expression, expected: &Expression) -> bool {
    let (e1, e2) = (actual.borrow_expr(), expected.borrow_expr());
    match e1 {
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
        e2 => e1 == e2,
    }
}

fn check_expression(input: &str, expected: Expression) {
    let actual = parse_expression(input).unwrap();
    assert!(
        expr_eq(&actual, &expected),
        "\nActual  : {:?}\nExpected: {:?}\n",
        actual,
        expected
    );
}

fn str_id(id: &str) -> Identifier {
    Identifier::new(id.into()).unwrap()
}

fn x() -> Identifier {
    str_id("x")
}

fn y() -> Identifier {
    str_id("y")
}

fn x_expr() -> Expression {
    B.id(x())
}

fn y_expr() -> Expression {
    B.id(y())
}

#[test]
fn int_literal_expr() {
    check_expression("1234", B.lit_i64(1234));
}

#[test]
fn add1() {
    check_expression("1+2", B.add_i64(1, 2));
}

#[test]
fn add2() {
    check_expression("1 + 2", B.add_i64(1, 2));
}

#[test]
fn add3() {
    check_expression("\t1 + 2", B.add_i64(1, 2));
}

#[test]
fn add4() {
    check_expression("1 + 2 + 3", B.add(B.add_i64(1, 2), B.lit_i64(3)));
}

#[test]
fn add5() {
    check_expression(
        "1 +\t2 + 3\n+ 4",
        B.add(B.add(B.add_i64(1, 2), B.lit_i64(3)), B.lit_i64(4)),
    );
}

#[test]
fn sub1() {
    check_expression("1 - 2 + 1", B.add(B.sub_i64(1, 2), B.lit_i64(1)));
}

#[test]
fn lit_type() {
    check_expression("  True ", B.lit_type_str("True"));
}

#[test]
fn add_id() {
    check_expression("1 +x", B.add(B.lit_i64(1), x_expr()));
}

#[test]
fn assignment1() {
    check_expression("val x = 1", B.assignment(str_id("x"), B.lit_i64(1)));
}

#[test]
fn assignment2() {
    check_expression(
        "val x = y + 2",
        B.assignment(str_id("x"), B.add(y_expr(), B.lit_i64(2))),
    );
}
