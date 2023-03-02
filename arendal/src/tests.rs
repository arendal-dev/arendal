use twi::value::Value;

use super::eval;

fn eval_ok(input: &str, result: Value) {
    if let Ok(v) = eval(input) {
        assert_eq!(v, result);
    } else {
        panic!("Error evaluating expression");
    }
}

fn eval_i64(input: &str, result: i64) {
    eval_ok(input, Value::int64(result))
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
