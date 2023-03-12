use core::error::Loc;
use core::typed::{TExprBuilder, TypedExpr};
use core::types::Type;

use super::{eval, Value};

const B: TExprBuilder = TExprBuilder::new(Loc::none());

fn eval_ok(input: TypedExpr, result: Value) {
    if let Ok(v) = eval(input) {
        assert_eq!(v, result);
    } else {
        panic!("Error evaluating expression");
    }
}

#[test]
fn integer() {
    eval_ok(B.lit_i64(1234), Value::int64(1234));
}

#[test]
fn add1() {
    eval_ok(B.add_i64(1, 2), Value::int64(3));
}

#[test]
fn add2() {
    eval_ok(
        B.add(Type::integer(), B.lit_i64(3), B.add_i64(1, 2)),
        Value::int64(6),
    );
}

#[test]
fn add_sub() {
    eval_ok(
        B.sub(Type::integer(), B.lit_i64(3), B.add_i64(1, 2)),
        Value::int64(0),
    );
}
