use super::{check, Expression, Type};
use crate::ast::ExprBuilder;
use crate::env::EnvRef;

const B: ExprBuilder = ExprBuilder::none();

fn ok_type(expr: Expression, t: Type) {
    let mut module = EnvRef::new_with_prelude().empty_local_module().unwrap();
    assert_eq!(*check(&mut module, &expr).unwrap().borrow_type(), t);
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
