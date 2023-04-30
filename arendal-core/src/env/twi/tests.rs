use crate::env::Env;
use crate::error::Result;
use crate::symbol::Pkg;

use super::Value;

fn eval_module(input: &str) -> Result<Value> {
    let parsed = crate::parser::parse(input)?;
    let mut env = Env::default();
    let path = Pkg::Local.empty();
    let module = crate::env::typecheck::check(&env, &path, &parsed)?;
    super::interpret(&mut env, &module)
}

fn eval_ok(input: &str, result: Value) {
    match eval_module(input) {
        Ok(value) => assert_eq!(value, result),
        Err(e) => panic!("Error evaluating expression {:?}", e),
    }
}

fn eval_i64(input: &str, result: i64) {
    eval_ok(input, Value::int64(result))
}

#[test]
fn empty1() {
    eval_ok("", Value::None);
}

#[test]
fn empty2() {
    eval_ok(" \t\n", Value::None);
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
