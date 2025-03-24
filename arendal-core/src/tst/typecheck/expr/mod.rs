use crate::ast0::{self, BinaryOp, Q};
use crate::context::Type;
use crate::error::{Error, Errors, L, Result};
use crate::symbol::{Symbol, TSymbol};
use crate::tst::Assignment;

use super::{Builder, Expr, Resolved, Scope, Value};

pub(super) fn check(scope: &Scope, input: &ast0::ExprRef) -> Result<L<Expr>> {
    match &input.it {
        ast0::Expr::LitInteger(value) => Ok(builder(input).val_integer(value.clone())),
        ast0::Expr::Symbol(q) => resolve_symbol(scope, input, &q),
        ast0::Expr::TSymbol(q) => {
            let tipo = resolve_type(scope, input, &q)?;
            let value = Value::v_singleton(&input.loc, tipo)?;
            Ok(builder(&input).value(value))
        }
        ast0::Expr::Seq(s) => {
            merge2(scope, &s.expr, &s.then).and_then(|(e1, e2)| Ok(builder(input).seq(e1, e2)))
        }
        ast0::Expr::Conditional(c) => {
            let (expr, then, otherwise) = Error::merge3(
                check(scope, &c.expr),
                check(scope, &c.then),
                check(scope, &c.otherwise),
            )?;
            builder(&input).conditional(expr, then, otherwise)
        }
        ast0::Expr::Binary(b) => merge2(scope, &b.expr1, &b.expr2)
            .and_then(|(t1, t2)| check_binary(scope, input, b.op, t1, t2)),
        ast0::Expr::Block(b) => {
            let result = check_block(scope, input, &b);
            result
        }
        _ => error(input, Error::InvalidType),
    }
}

fn resolve_type(scope: &Scope, input: &ast0::ExprRef, symbol: &Q<TSymbol>) -> Result<Type> {
    scope.resolve_type(&input.loc, symbol)
}

fn resolve_symbol(scope: &Scope, input: &ast0::ExprRef, symbol: &Q<Symbol>) -> Result<L<Expr>> {
    Ok(match scope.resolve_symbol(&input.loc, symbol)? {
        Resolved::Local(local) => builder(input).local0(local),
        Resolved::Global(global) => builder(input).global0(global),
    })
}

fn check_binary(
    scope: &Scope,
    input: &ast0::ExprRef,
    op: BinaryOp,
    expr1: L<Expr>,
    expr2: L<Expr>,
) -> Result<L<Expr>> {
    match op {
        BinaryOp::Add => builder(input).int_add(expr1, expr2),
        BinaryOp::Sub => builder(input).int_sub(expr1, expr2),
        BinaryOp::Mul => builder(input).int_mul(expr1, expr2),
        BinaryOp::Div => builder(input).int_div(expr1, expr2),
        BinaryOp::And => builder(input).log_and(expr1, expr2),
        BinaryOp::Or => builder(input).log_or(expr1, expr2),
        _ => error(input, Error::InvalidType),
    }
}

fn check_block(scope: &Scope, input: &ast0::ExprRef, block: &ast0::Block) -> Result<L<Expr>> {
    let mut child_scope = scope.child();
    let assignments = check_assignments(&mut child_scope, &block.assignments)?;
    let mut expr: Option<L<Expr>> = None;
    for e in &block.exprs {
        if expr.is_none() {
            expr = Some(check(&child_scope, &e)?)
        } else {
            return error(input, Error::OnlyOneExpressionAllowed);
        }
    }
    builder(&input).block(assignments, expr)
}

fn check_assignments(
    scope: &mut Scope,
    ast: &Vec<ast0::LAssignmentRef>,
) -> Result<Vec<L<Assignment>>> {
    add_assignment_candidates(scope, ast)?;
    let mut assignments = Vec::default();
    loop {
        let mut errors = Errors::default();
        let progress = assignments.len();
        for a in ast {
            if scope.eval_pending(&a.it.symbol) {
                errors
                    .add_result(check_assignment(scope, a))
                    .map(|a| assignments.push(a));
            }
        }
        match errors.to_unit_result() {
            Ok(_) => return Ok(assignments),
            Err(e) => {
                let missing = e.missing_local_symbol_deps();
                if assignments.len() == progress || missing.is_empty() {
                    return Err(e);
                }
            }
        }
    }
}

fn add_assignment_candidates(scope: &mut Scope, ast: &Vec<ast0::LAssignmentRef>) -> Result<()> {
    let mut errors = Errors::default();
    for a in ast {
        errors.add_result(scope.add_current(&a.loc, a.it.symbol.clone()));
    }
    errors.to_unit_result()
}

fn check_assignment(scope: &mut Scope, a: &L<ast0::Assignment>) -> Result<L<Assignment>> {
    let typed = check(scope, &a.it.expr)?;
    scope.set(&a.loc, a.it.symbol.clone(), typed.get_type())?;
    Ok(Builder::new(a.loc.clone()).assignment(a.it.symbol.clone(), typed))
}

fn merge2(scope: &Scope, e1: &ast0::ExprRef, e2: &ast0::ExprRef) -> Result<(L<Expr>, L<Expr>)> {
    Error::merge(check(scope, e1), check(scope, e2))
}

fn builder(input: &ast0::ExprRef) -> Builder {
    Builder::new(input.loc.clone())
}

// Creates and returns an error
fn error(input: &ast0::ExprRef, error: Error) -> Result<L<Expr>> {
    input.loc.err(error)
}
