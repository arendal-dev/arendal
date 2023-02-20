use super::{check, Expression, Type};
use ast::helper::*;

fn ok_type(expr: Expression, t: Type) {
    assert_eq!(*check(expr).unwrap().borrow_type(), t);
}

fn ok_int(expr: Expression) {
    ok_type(expr, Type::Integer);
}

#[test]
fn integer() {
    ok_int(lit_i64(1234));
}

#[test]
fn sum1() {
    ok_int(add_i64(1, 2));
}

#[test]
fn sum2() {
    ok_int(add(add_i64(1, 2), lit_i64(3)));
}
