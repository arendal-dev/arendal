use super::{check, Expression, Type};
use crate::ast::helper::*;
use crate::scope::Scope;

fn ok_type(expr: Expression, t: Type) {
    assert_eq!(
        *check(&mut Scope::builtin(), &expr).unwrap().borrow_type(),
        t
    );
}

fn ok_int(expr: Expression) {
    ok_type(expr, Type::integer());
}

#[test]
fn integer() {
    ok_int(lit_i64(1234));
}

#[test]
fn add1() {
    ok_int(add_i64(1, 2));
}

#[test]
fn add2() {
    ok_int(add(add_i64(1, 2), lit_i64(3)));
}

#[test]
fn sub1() {
    ok_int(sub_i64(1, 2));
}
