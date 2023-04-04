use crate::ast::{BinaryOp, Expr, Expression};
use crate::error::{Errors, Result};
use crate::typed::{TExprBuilder, TypedExpr};
use crate::types::Type;

use super::{Module, TypeError};

fn builder(input: &Expression) -> TExprBuilder {
    TExprBuilder::new(input.clone_loc())
}

pub(super) fn check(module: &mut Module, input: &Expression) -> Result<TypedExpr> {
    match input.borrow_expr() {
        Expr::LitInteger(value) => Ok(builder(input).lit_integer(value.clone())),
        Expr::Symbol(id) => match module.get_val(id) {
            Some(tipo) => Ok(builder(input).val(id.clone(), tipo.clone())),
            None => error(input, TypeError::UnknownIdentifier(id.clone())),
        },
        Expr::Assignment(id, expr) => {
            let typed = check(module, expr)?;
            module.add_val(input.clone_loc(), id.clone(), typed.clone_type())?;
            Ok(builder(input).assignment(id.clone(), typed))
        }
        Expr::Binary(op, e1, e2) => {
            Errors::merge(check(module, e1), check(module, e2), |t1, t2| {
                check_binary(module, input, *op, t1, t2)
            })
        }
        _ => error(input, TypeError::InvalidType),
    }
}

// Creates and returns an error
fn error(input: &Expression, kind: TypeError) -> Result<TypedExpr> {
    Errors::err(input.clone_loc(), kind)
}

fn check_binary(
    module: &Module,
    input: &Expression,
    op: BinaryOp,
    e1: TypedExpr,
    e2: TypedExpr,
) -> Result<TypedExpr> {
    match op {
        BinaryOp::Add => check_add(module, input, e1, e2),
        BinaryOp::Sub => check_sub(module, input, e1, e2),
        BinaryOp::Mul => check_mul(module, input, e1, e2),
        BinaryOp::Div => check_div(module, input, e1, e2),
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
    Ok(builder(input).binary(tipo, op, e1, e2))
}

fn check_add(
    module: &Module,
    input: &Expression,
    e1: TypedExpr,
    e2: TypedExpr,
) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::Integer, BinaryOp::Add, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

fn check_sub(
    module: &Module,
    input: &Expression,
    e1: TypedExpr,
    e2: TypedExpr,
) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::Integer, BinaryOp::Sub, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

fn check_mul(
    module: &Module,
    input: &Expression,
    e1: TypedExpr,
    e2: TypedExpr,
) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::Integer, BinaryOp::Mul, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

fn check_div(
    module: &Module,
    input: &Expression,
    e1: TypedExpr,
    e2: TypedExpr,
) -> Result<TypedExpr> {
    if e1.is_integer() && e2.is_integer() {
        ok_binary(input, Type::Integer, BinaryOp::Div, e1, e2)
    } else {
        error(input, TypeError::InvalidType)
    }
}

#[cfg(test)]
mod tests;
