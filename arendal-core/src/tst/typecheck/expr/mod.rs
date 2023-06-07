use crate::ast::{self, BinaryOp, Q};
use crate::error::{Error, Result, L};
use crate::symbol::{FQPath, Symbol, TSymbol};
use crate::types::Type;

use super::{BStmt, Builder, Expr, Scope, TypeChecker, Value};

pub(super) fn check<'a>(
    checker: &TypeChecker<'a>,
    path: &FQPath,
    scope: &Scope,
    input: &L<ast::Expr>,
) -> Result<L<Expr>> {
    ExprChecker {
        checker,
        path,
        scope,
        input,
    }
    .check()
}

#[derive(Debug)]
struct ExprChecker<'a, 'b> {
    checker: &'b TypeChecker<'a>,
    path: &'b FQPath,
    scope: &'b Scope,
    input: &'b L<ast::Expr>,
}

impl<'a, 'b> ExprChecker<'a, 'b> {
    fn merge2(&self, e1: &L<ast::Expr>, e2: &L<ast::Expr>) -> Result<(L<Expr>, L<Expr>)> {
        Error::merge(self.sub_expr(&e1), self.sub_expr(&e2))
    }

    fn check(self) -> Result<L<Expr>> {
        match &self.input.it {
            ast::Expr::LitInteger(value) => Ok(self.builder().val_integer(value.clone())),
            ast::Expr::Symbol(q) => self.check_symbol(q),
            ast::Expr::TSymbol(q) => {
                let tipo = self.resolve_type(&q)?;
                let value = Value::singleton(&self.input.loc, &tipo)?;
                Ok(self.builder().value(value))
            }
            ast::Expr::Seq(s) => self
                .merge2(&s.expr, &s.then)
                .and_then(|(e1, e2)| Ok(self.builder().seq(e1, e2))),
            ast::Expr::Conditional(c) => {
                let (expr, then, otherwise) = Error::merge3(
                    self.sub_expr(&c.expr),
                    self.sub_expr(&c.then),
                    self.sub_expr(&c.otherwise),
                )?;
                self.builder().conditional(expr, then, otherwise)
            }
            ast::Expr::Binary(b) => self
                .merge2(&b.expr1, &b.expr2)
                .and_then(|(t1, t2)| self.check_binary(b.op, t1, t2)),
            ast::Expr::Block(v) => {
                let result = self.check_block(v);
                result
            }
            _ => self.error(Error::InvalidType),
        }
    }

    fn resolve_type(&self, symbol: &Q<TSymbol>) -> Result<Type> {
        self.checker
            .resolve_type(&self.input.loc, self.path, symbol)
    }

    fn sub_expr(&self, input: &L<ast::Expr>) -> Result<L<Expr>> {
        check(self.checker, self.path, &self.scope, input)
    }

    fn check_symbol(self, q: &Q<Symbol>) -> Result<L<Expr>> {
        Ok(if q.segments.is_empty() && self.scope.contains(&q.symbol) {
            self.builder()
                .local(q.symbol.clone(), self.scope.get(&q.symbol).unwrap().clone())
        } else {
            self.builder()
                .global0(self.checker.resolve_global(&self.input.loc, self.path, q)?)
        })
    }

    fn check_binary(self, op: BinaryOp, expr1: L<Expr>, expr2: L<Expr>) -> Result<L<Expr>> {
        match op {
            BinaryOp::Add => self.builder().int_add(expr1, expr2),
            BinaryOp::Sub => self.builder().int_sub(expr1, expr2),
            BinaryOp::Mul => self.builder().int_mul(expr1, expr2),
            BinaryOp::Div => self.builder().int_div(expr1, expr2),
            BinaryOp::And => self.builder().log_and(expr1, expr2),
            BinaryOp::Or => self.builder().log_or(expr1, expr2),
            _ => self.error(Error::InvalidType),
        }
    }

    fn check_block(self, stmts: &Vec<ast::BStmt>) -> Result<L<Expr>> {
        let mut child_scope = self.scope.create_child();
        let mut checked = Vec::default();
        for s in stmts {
            match s {
                ast::BStmt::Assignment(a) => {
                    checked.push(self.check_assignment(&mut child_scope, a.as_ref())?)
                }
                ast::BStmt::Expr(e) => checked
                    .push(check(self.checker, self.path, &child_scope, e.as_ref())?.to_stmt()),
            }
        }
        self.builder().block(checked)
    }

    fn check_assignment(&self, scope: &mut Scope, a: &L<ast::Assignment>) -> Result<L<BStmt>> {
        let typed = self.sub_expr(&a.it.expr)?;
        scope.set(&a.loc, a.it.symbol.clone(), typed.clone_type())?;
        Ok(self.builder().assignment(a.it.symbol.clone(), typed))
    }

    fn builder(&self) -> Builder {
        Builder::new(self.input.loc.clone())
    }

    // Creates and returns an error
    fn error(self, error: Error) -> Result<L<Expr>> {
        self.input.loc.err(error)
    }
}
