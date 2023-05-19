use im::HashMap;

use crate::ast::{self, BinaryOp};
use crate::error::{Error, Result};
use crate::symbol::{Symbol, TSymbol};
use crate::types::Type;

use super::{ExprBuilder, Expression, ModuleChecker, Value};

type Scope = HashMap<Symbol, Type>;

pub(super) fn check<'a>(
    checker: &mut ModuleChecker<'a>,
    input: &ast::Expression,
) -> Result<Expression> {
    ExprChecker { checker, input }.check()
}

#[derive(Debug)]
struct ExprChecker<'a, 'b> {
    checker: &'b mut ModuleChecker<'a>,
    input: &'b ast::Expression,
}

impl<'a, 'b> ExprChecker<'a, 'b> {
    fn check(mut self) -> Result<Expression> {
        match &self.input.expr {
            ast::Expr::LitInteger(value) => Ok(self.builder().val_integer(value.clone())),
            ast::Expr::Symbol(id) => match self.checker.get_val(&id) {
                Some(tipo) => Ok(self.builder().local(id.clone(), tipo.clone())),
                None => self.error(Error::UnknownLocalSymbol(id.clone())),
            },
            ast::Expr::TSymbol(s) => {
                let tipo = self.resolve_type(&s)?;
                let value = Value::singleton(&self.input.loc, &tipo)?;
                Ok(self.builder().value(value))
            }
            ast::Expr::Conditional(c) => {
                let (expr, then, otherwise) = Error::merge3(
                    self.sub_expr(&c.expr),
                    self.sub_expr(&c.then),
                    self.sub_expr(&c.otherwise),
                )?;
                self.builder().conditional(expr, then, otherwise)
            }
            ast::Expr::Assignment(a) => {
                let typed = self.sub_expr(&a.expr)?;
                self.checker.set_val(
                    self.input.loc.clone(),
                    a.symbol.clone(),
                    typed.clone_type(),
                )?;
                Ok(self.builder().assignment(a.symbol.clone(), typed))
            }
            ast::Expr::Binary(b) => Error::merge(self.sub_expr(&b.expr1), self.sub_expr(&b.expr2))
                .and_then(|(t1, t2)| self.check_binary(b.op, t1, t2)),
            ast::Expr::Block(v) => {
                self.checker.scopes.push(Scope::default());
                let result = self.check_block(v);
                self.checker.scopes.pop();
                result
            }
            _ => self.error(Error::InvalidType),
        }
    }

    fn resolve_type(&self, symbol: &TSymbol) -> Result<Type> {
        self.checker.resolve_type(&self.input.loc, symbol)
    }

    fn sub_expr(&mut self, input: &ast::Expression) -> Result<Expression> {
        ExprChecker {
            checker: self.checker,
            input,
        }
        .check()
    }

    fn check_binary(
        self,
        op: BinaryOp,
        expr1: Expression,
        expr2: Expression,
    ) -> Result<Expression> {
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

    fn check_block(&mut self, exprs: &Vec<ast::Expression>) -> Result<Expression> {
        let mut checked = Vec::default();
        for e in exprs {
            checked.push(self.sub_expr(e)?);
        }
        self.builder().block(checked)
    }

    fn builder(&self) -> ExprBuilder {
        ExprBuilder::new(self.input.loc.clone())
    }

    // Creates and returns an error
    fn error(self, error: Error) -> Result<Expression> {
        self.input.loc.err(error)
    }
}