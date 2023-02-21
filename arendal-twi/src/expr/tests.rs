use super::{eval, Value};
use ast::typed::helper::*;
use ast::{Type, TypedExpr};

fn eval_ok(input: TypedExpr, result: Value) {
    if let Ok(v) = eval(input) {
        assert_eq!(v, result);
    } else {
        panic!("Error evaluating expression");
    }
}

#[test]
fn integer() {
    eval_ok(lit_i64(1234), Value::int64(1234));
}

#[test]
fn add1() {
    eval_ok(add_i64(1, 2), Value::int64(3));
}

#[test]
fn add2() {
    eval_ok(
        add(Type::Integer, lit_i64(3), add_i64(1, 2)),
        Value::int64(6),
    );
}

#[test]
fn add_sub() {
    eval_ok(
        sub(Type::Integer, lit_i64(3), add_i64(1, 2)),
        Value::int64(0),
    );
}
