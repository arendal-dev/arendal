use crate::env::Env;
use crate::error::{Loc, Result};
use crate::symbol::Pkg;

use super::Value;

fn eval_module(input: &str) -> Result<Value> {
    let mut env = Env::default();
    let path = Pkg::Local.empty();
    super::super::run(&mut env, input)
}

fn eval_ok(input: &str, result: Value) {
    match eval_module(input) {
        Ok(value) => assert_eq!(value, result),
        Err(e) => panic!("Error evaluating expression {:?}", e),
    }
}

fn v_none() -> Value {
    Value::v_none(&Loc::None)
}

fn v_true() -> Value {
    Value::v_true(&Loc::None)
}

fn v_false() -> Value {
    Value::v_true(&Loc::None)
}

fn v_i64(value: i64) -> Value {
    Value::integer(&Loc::None, value.into())
}

fn eval_i64(input: &str, result: i64) {
    eval_ok(input, v_i64(result))
}

#[test]
fn empty1() {
    eval_ok("", v_none());
}

#[test]
fn empty2() {
    eval_ok(" \t\n", v_none());
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
    eval_ok("(True || False) && True", v_true());
}

#[test]
fn conditional1() {
    eval_i64("if True then 1 else 2", 1);
}

#[test]
fn block() {
    eval_i64("let x = 1\n{ let y = x + 1\n y + 2\n}", 4);
}
