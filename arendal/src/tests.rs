use core::{
    context::{Type, Value},
    error::Loc,
};

fn eval_ok(input: &str, result: Value) {
    match super::REPL::new().eval(input) {
        Ok(v) => {
            assert_eq!(v, result);
        }
        _ => {
            panic!("Error evaluating expression");
        }
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
fn add() {
    eval_i64("1+2", 3);
}
