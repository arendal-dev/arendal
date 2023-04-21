use super::{Expression, Type, TypeChecker};
use crate::ast::ExprBuilder;
use crate::env::Env;
use crate::symbol::{Path, Pkg};

const B: ExprBuilder = ExprBuilder::none();

fn ok_type(expr: Expression, t: Type) {
    let env = Env::default();
    let mut checker = TypeChecker::new(&env, Pkg::Local, Path::empty());
    assert_eq!(*checker.expression(&expr).unwrap().borrow_type(), t);
}

fn ok_int(expr: Expression) {
    ok_type(expr, Type::Integer);
}

#[test]
fn integer() {
    ok_int(B.lit_i64(1234));
}

#[test]
fn add1() {
    ok_int(B.add_i64(1, 2));
}

#[test]
fn add2() {
    ok_int(B.add(B.add_i64(1, 2), B.lit_i64(3)));
}

#[test]
fn sub1() {
    ok_int(B.sub_i64(1, 2));
}
