use crate::ast::BinaryOp;
use crate::error::Errors;
use crate::typed::TExpr::*;
use crate::typed::TypedExpr;
use crate::value::Value;
use crate::Integer;

use super::{RuntimeError, ValueResult};

use super::Interpreter;

fn integer(value: Integer) -> ValueResult {
    Ok(Value::Int(value))
}

fn err(expr: &TypedExpr, error: RuntimeError) -> ValueResult {
    Errors::err(expr.clone_loc(), error)
}

pub(crate) fn eval(interpreter: &mut Interpreter, expr: &TypedExpr) -> ValueResult {
    match expr.borrow_expr() {
        LitInteger(i) => integer(i.clone()),
        Val(id) => match interpreter.get_val(id) {
            Some(value) => Ok(value),
            None => err(expr, RuntimeError::UknownVal(id.clone())),
        },
        Assignment(id, expr) => {
            let value = eval(interpreter, expr)?;
            interpreter.set_val(id.clone(), value.clone());
            Ok(value)
        }
        Binary(op, e1, e2) => binary(interpreter, *op, e1, e2),
        _ => err(expr, RuntimeError::NotImplemented),
    }
}

fn binary(
    interpreter: &mut Interpreter,
    op: BinaryOp,
    e1: &TypedExpr,
    e2: &TypedExpr,
) -> ValueResult {
    let v1 = eval(interpreter, e1)?;
    match op {
        BinaryOp::Add => add(interpreter, v1, e2),
        BinaryOp::Sub => sub(interpreter, v1, e2),
        BinaryOp::Mul => mul(interpreter, v1, e2),
        BinaryOp::Div => div(interpreter, v1, e2),
        _ => err(e1, RuntimeError::NotImplemented),
    }
}

fn add(interpreter: &mut Interpreter, v1: Value, e2: &TypedExpr) -> ValueResult {
    let v2 = eval(interpreter, e2)?;
    // We only have integers for now
    integer(v1.as_integer().unwrap() + v2.as_integer().unwrap())
}

fn sub(interpreter: &mut Interpreter, v1: Value, e2: &TypedExpr) -> ValueResult {
    let v2 = eval(interpreter, e2)?;
    // We only have integers for now
    integer(v1.as_integer().unwrap() - v2.as_integer().unwrap())
}

fn mul(interpreter: &mut Interpreter, v1: Value, e2: &TypedExpr) -> ValueResult {
    let v2 = eval(interpreter, e2)?;
    // We only have integers for now
    integer(v1.as_integer().unwrap() * v2.as_integer().unwrap())
}

fn div(interpreter: &mut Interpreter, v1: Value, e2: &TypedExpr) -> ValueResult {
    let v2 = eval(interpreter, e2)?;
    // We only have integers for now
    let i2 = v2.as_integer().unwrap();
    if i2.is_zero() {
        err(e2, RuntimeError::DivisionByZero)
    } else {
        integer(v1.as_integer().unwrap() / i2)
    }
}

#[cfg(test)]
mod tests;
