use super::eval;
use ast::Type;
use num::Integer;
use twi::{TypedValue, Value};

fn eval_ok(input: &str, result: TypedValue) {
    if let Ok(v) = eval(input) {
        assert_eq!(v, result);
    } else {
        panic!("Error evaluating expression");
    }
}

#[test]
fn integer() {
    eval_ok("1234", TypedValue::int64(1234));
}
