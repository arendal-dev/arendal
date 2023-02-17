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
fn sum1() {
    eval_ok("1+2", Value::int64(3));
}

#[test]
fn sum2() {
    eval_ok("1 + 2 + 3", Value::int64(6));
}
