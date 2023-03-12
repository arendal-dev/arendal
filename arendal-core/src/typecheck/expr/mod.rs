use crate::ast::{BinaryOp, Expr, Expression};
use crate::error::{Errors, Result};
use crate::names::Names;
use crate::typed::TypedExpr;
use crate::types::Type;

use super::TypeError;

pub(super) fn check(names: &mut Names, input: &Expression) -> Result<TypedExpr> {
    match input.borrow_expr() {
        Expr::LitInteger(value) => Ok(TypedExpr::lit_integer(input.clone_loc(), value.clone())),
        Expr::Id(id) => {
            match names.get_val(id) {
                Some(tipo) => Ok(TypedExpr::val(input.clone_loc(), id.clone(), tipo.clone())),
                None => error(input, TypeError::UnknownIdentifier(id.clone())),
                
            }
        }
        Expr::Binary(op, e1, e2) => Errors::merge(check(names, e1), check(names, e2), |t1, t2| {
            check_binary(names, input, *op, t1, t2)
        }),
        _ => error(input, TypeError::InvalidType),
    }
}

// Creates and returns an error
fn error(input: &Expression, kind: TypeError) -> Result<TypedExpr> {
    Errors::err(input.clone_loc(), kind)
}

fn check_binary(
    names: &Names,
    input: &Expression,
    op: BinaryOp,
    e1: TypedExpr,
    e2: TypedExpr,
) -> Result<TypedExpr> {
    match op {
        BinaryOp::Add => check_add(names, input, e1, e2),
        BinaryOp::Sub => check_sub(names, input, e1, e2),
        BinaryOp::Mul => check_mul(names, input, e1, e2),
        BinaryOp::Div => check_div(names, input, e1, e2),
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

fn check_add(names: &Names, input: &Expression, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::integer(), BinaryOp::Add, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

fn check_sub(names: &Names, input: &Expression, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::integer(), BinaryOp::Sub, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

fn check_mul(names: &Names, input: &Expression, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::integer(), BinaryOp::Mul, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

fn check_div(names: &Names, input: &Expression, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::integer(), BinaryOp::Div, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

#[cfg(test)]
mod tests;
