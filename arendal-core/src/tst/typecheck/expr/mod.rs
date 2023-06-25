use crate::ast::{self, BinaryOp, Q};
use crate::error::{Error, Result, L};
use crate::symbol::{Symbol, TSymbol};
use crate::tst::Assignment;
use crate::types::Type;

use super::{Builder, Expr, Resolved, Scope, Value};

pub(super) fn check<'a, 'b, 'c>(
    scope: &mut Scope<'a, 'b, 'c>,
    input: &L<ast::Expr>,
) -> Result<L<Expr>> {
    ExprChecker { scope, input }.check()
}

struct ExprChecker<'a, 'b, 'c, 'd> {
    scope: &'d mut Scope<'a, 'b, 'c>,
    input: &'d L<ast::Expr>,
}

impl<'a, 'b, 'c, 'd> ExprChecker<'a, 'b, 'c, 'd> {
    fn merge2(&mut self, e1: &L<ast::Expr>, e2: &L<ast::Expr>) -> Result<(L<Expr>, L<Expr>)> {
        Error::merge(self.sub_expr(&e1), self.sub_expr(&e2))
    }

    fn check(mut self) -> Result<L<Expr>> {
        match &self.input.it {
            ast::Expr::LitInteger(value) => Ok(self.builder().val_integer(value.clone())),
            ast::Expr::Symbol(q) => self.resolve_symbol(q),
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
            ast::Expr::Block(b) => {
                let result = self.check_block(b.as_ref());
                result
            }
            _ => self.error(Error::InvalidType),
        }
    }

    fn resolve_type(&self, symbol: &Q<TSymbol>) -> Result<Type> {
        self.scope.resolve_type(&self.input.loc, symbol)
    }

    fn resolve_symbol(self, symbol: &Q<Symbol>) -> Result<L<Expr>> {
        Ok(match self.scope.resolve_symbol(&self.input.loc, symbol)? {
            Resolved::Local(local) => self.builder().local0(local),
            Resolved::Global(global) => self.builder().global0(global),
        })
    }

    fn sub_expr(&mut self, input: &L<ast::Expr>) -> Result<L<Expr>> {
        check(self.scope, input)
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

    fn check_block(mut self, block: &ast::Block) -> Result<L<Expr>> {
        let mut assignments = Vec::default();
        let mut child_scope = self.scope.child();
        let mut expr: Option<L<Expr>> = None;
        for a in &block.assignments {
            assignments.push(self.check_assignment(&mut child_scope, a)?)
        }
        for e in &block.exprs {
            if expr.is_none() {
                expr = Some(check(&mut child_scope, e)?)
            } else {
                return self.error(Error::OnlyOneExpressionAllowed);
            }
        }
        self.builder().block(assignments, expr)
    }

    fn check_assignment(
        &mut self,
        scope: &mut Scope,
        a: &L<ast::Assignment>,
    ) -> Result<L<Assignment>> {
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
