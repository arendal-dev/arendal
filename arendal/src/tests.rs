use super::eval;
use twi::Value;

fn eval_ok(input: &str, result: Value) {
    if let Ok(v) = eval(input) {
        assert_eq!(v, result);
    } else {
        panic!("Error evaluating expression");
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
    eval_ok("1 + 2 + 3", Value::int64(6));
}

#[test]
fn sub1() {
    eval_ok("1 - 2", Value::int64(-1));
}

#[test]
fn mul1() {
    eval_ok("1 + 2 * 2", Value::int64(5));
}
