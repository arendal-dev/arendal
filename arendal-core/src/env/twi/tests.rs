use crate::env::Env;
use crate::error::{Loc, Result};
use crate::types::Type;

use super::Value;

fn eval_module(input: &str) -> Result<Value> {
    let mut env = Env::default();
    env.run(input)
}

fn eval_ok(input: &str, result: Value) {
    match eval_module(input) {
        Ok(value) => assert_eq!(value, result),
        Err(e) => panic!("Error evaluating expression {:?}", e),
    }
}

fn v_i64(value: i64) -> Value {
    Value::v_integer(&Loc::None, Type::type_integer(), value.into()).unwrap()
}

fn eval_i64(input: &str, result: i64) {
    eval_ok(input, v_i64(result))
}

#[test]
fn empty1() {
    eval_ok("", Value::v_none());
}

#[test]
fn empty2() {
    eval_ok(" \t\n", Value::v_none());
}

#[test]
fn integer() {
    eval_i64("1234", 1234);
}

#[test]
fn add1() {
    eval_i64("1+2", 3);
}

#[test]
fn add2() {
    eval_i64("1 + 2 + 3", 6);
}

#[test]
fn sub1() {
    eval_i64("1 - 2", -1);
}

#[test]
fn mul1() {
    eval_i64("1 + 2 * 2", 5);
}

#[test]
fn parens1() {
    eval_i64("(1 + 2) * 2", 6);
}

#[test]
fn parens2() {
    eval_i64("(((1 + 2) * 2) + 1) * 2", 14);
}

#[test]
fn logical() {
    eval_ok("(True || False) && True", Value::v_true());
}

#[test]
fn conditional1() {
    eval_i64("if True then 1 else 2", 1);
}

#[test]
fn seq() {
    eval_i64("1 then 2", 2);
    eval_i64("1 then 2 then 3", 3);
}

#[test]
fn assignments() {
    eval_i64("let a = b + c\nlet b = c\nlet c = 3\na - 1", 5);
}

#[test]
fn block() {
    eval_i64("let x = 1\n{ let y = x + 1\n y + 2\n}", 4);
}
