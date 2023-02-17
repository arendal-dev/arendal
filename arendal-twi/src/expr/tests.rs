use super::{eval, Value};
use ast::bare::BareLoc;
use ast::{BinaryOp, Expression, Type, TypedExpression, TypedLoc};

type Typed = TypedExpression<BareLoc>;

fn loc(t: Type) -> TypedLoc<BareLoc> {
    TypedLoc {
        loc: BareLoc {},
        loc_type: t,
    }
}

fn lit_i64(value: i64) -> Typed {
    Expression::lit_integer(loc(Type::Integer), value.into())
}

fn binary(t: Type, op: BinaryOp, expr1: Typed, expr2: Typed) -> Typed {
    Expression::binary(loc(t), op, expr1, expr2)
}

fn add(t: Type, expr1: Typed, expr2: Typed) -> Typed {
    binary(t, BinaryOp::Add, expr1, expr2)
}

fn add_i64(v1: i64, v2: i64) -> Typed {
    add(Type::Integer, lit_i64(v1), lit_i64(v2))
}

fn eval_ok(input: Typed, result: Value) {
    if let Ok(v) = eval(input) {
        assert_eq!(v, result);
    } else {
        panic!("Error evaluating expression");
    }
}

#[test]
fn integer() {
    eval_ok(lit_i64(1234), Value::int64(1234));
}

#[test]
fn sum1() {
    eval_ok(add_i64(1, 2), Value::int64(3));
}

#[test]
fn sum2() {
    eval_ok(
        add(Type::Integer, lit_i64(3), add_i64(1, 2)),
        Value::int64(6),
    );
}
