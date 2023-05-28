use core::values::Value;

fn eval_ok(input: &str, result: Value) {
    if let Ok(v) = super::REPL::new().eval(input) {
        assert_eq!(v, result);
    } else {
        panic!("Error evaluating expression");
    }
}

fn v_i64(value: i64) -> Value {
    Value::Integer(value.into())
}

fn eval_i64(input: &str, result: i64) {
    eval_ok(input, v_i64(result))
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
fn add() {
    eval_i64("1+2", 3);
}
