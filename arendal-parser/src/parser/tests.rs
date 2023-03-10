use core::ast::helper::*;
use core::ast::Expr::*;
use core::ast::Expression;
use core::id::Id;

use super::parse_expression;

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

fn str_id(id: &str) -> Id {
    Id::new(id.into()).unwrap()
}

fn x() -> Id {
    str_id("x")
}

fn y() -> Id {
    str_id("y")
}
fn x_expr() -> Expression {
    id(x())
}

fn y_expr() -> Expression {
    id(y())
}

#[test]
fn int_literal_expr() {
    check_expression("1234", lit_i64(1234));
}

#[test]
fn add1() {
    check_expression("1+2", add_i64(1, 2));
}

#[test]
fn add2() {
    check_expression("1 + 2", add_i64(1, 2));
}

#[test]
fn add3() {
    check_expression("\t1 + 2", add_i64(1, 2));
}

#[test]
fn add4() {
    check_expression("1 + 2 + 3", add(add_i64(1, 2), lit_i64(3)));
}

#[test]
fn add5() {
    check_expression(
        "1 +\t2 + 3\n+ 4",
        add(add(add_i64(1, 2), lit_i64(3)), lit_i64(4)),
    );
}

#[test]
fn sub1() {
    check_expression("1 - 2 + 1", add(sub_i64(1, 2), lit_i64(1)));
}

#[test]
fn lit_type() {
    check_expression("  True ", lit_type_str("True"));
}

#[test]
fn add_id() {
    check_expression("1 +x", add(lit_i64(1), x_expr()));
}

#[test]
fn assignment1() {
    check_expression("val x = 1", assignment(str_id("x"), lit_i64(1)));
}

#[test]
fn assignment2() {
    check_expression(
        "val x = y + 2",
        assignment(str_id("x"), add(y_expr(), lit_i64(2))),
    );
}
