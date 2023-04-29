use crate::env::Env;
use crate::error::Loc;
use crate::symbol::Pkg;
use crate::typed::{ExprBuilder, Expression, Expressions, Module};

use super::Value;

const B: ExprBuilder = ExprBuilder::new(Loc::none());

fn eval_ok(input: Expression, result: Value) {
    let mut env = Env::default();
    let output = super::interpret(
        &mut env,
        &Module {
            path: Pkg::Local.empty(),
            expressions: Expressions::new(vec![input]),
        },
    );
    if let Ok(v) = output {
        assert_eq!(v, result);
    } else {
        panic!("Error evaluating expression");
    }
}

#[test]
fn integer() {
    eval_ok(B.val_i64(1234), Value::int64(1234));
}

#[test]
fn add1() {
    eval_ok(B.add_i64(1, 2), Value::int64(3));
}

#[test]
fn add2() {
    eval_ok(B.add(B.val_i64(3), B.add_i64(1, 2)), Value::int64(6));
}

#[test]
fn add_sub() {
    eval_ok(B.sub(B.val_i64(3), B.add_i64(1, 2)), Value::int64(0));
}
