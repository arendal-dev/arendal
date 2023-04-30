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

#[test]
fn integer() {
    eval_ok("1234", Value::int64(1234));
}

#[test]
fn add1() {
    eval_ok("1+2", Value::int64(3));
}

#[test]
fn add2() {
    eval_ok("3+1+2", Value::int64(6));
}

#[test]
fn add_sub() {
    eval_ok("3-(1+2)", Value::int64(0));
}
