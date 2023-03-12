use crate::ast::{BinaryOp, Expr, Expression};
use crate::error::{Errors, Result};
use crate::scope::Scope;
use crate::typed::TypedExpr;
use crate::types::Type;

use super::TypeError;

pub(super) fn check(scope: &mut Scope, input: &Expression) -> Result<TypedExpr> {
    match input.borrow_expr() {
        Expr::LitInteger(value) => Ok(TypedExpr::lit_integer(input.clone_loc(), value.clone())),
        Expr::Binary(op, e1, e2) => Errors::merge(check(scope, e1), check(scope, e2), |t1, t2| {
            check_binary(scope, input, *op, t1, t2)
        }),
        _ => error(input, TypeError::InvalidType),
    }
}

// Creates and returns an error
fn error(input: &Expression, kind: TypeError) -> Result<TypedExpr> {
    Err(Errors::new(input.clone_loc(), kind))
}

fn check_binary(
    scope: &Scope,
    input: &Expression,
    op: BinaryOp,
    e1: TypedExpr,
    e2: TypedExpr,
) -> Result<TypedExpr> {
    match op {
        BinaryOp::Add => check_add(scope, input, e1, e2),
        BinaryOp::Sub => check_sub(scope, input, e1, e2),
        BinaryOp::Mul => check_mul(scope, input, e1, e2),
        BinaryOp::Div => check_div(scope, input, e1, e2),
        _ => error(input, TypeError::InvalidType),
    }
}

fn ok_binary(
    input: &Expression,
    tipo: Type,
    op: BinaryOp,
    e1: TypedExpr,
    e2: TypedExpr,
) -> Result<TypedExpr> {
    Ok(TypedExpr::binary(input.clone_loc(), tipo, op, e1, e2))
}

fn check_add(scope: &Scope, input: &Expression, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::integer(), BinaryOp::Add, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

fn check_sub(scope: &Scope, input: &Expression, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::integer(), BinaryOp::Sub, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

fn check_mul(scope: &Scope, input: &Expression, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::integer(), BinaryOp::Mul, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

fn check_div(scope: &Scope, input: &Expression, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::integer(), BinaryOp::Div, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

#[cfg(test)]
mod tests;
