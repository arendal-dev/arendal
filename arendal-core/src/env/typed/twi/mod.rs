use super::{Expr, Expression, Module};
use crate::error::{Error, Loc, Result};
use crate::symbol::Symbol;
use crate::types::Type;
use crate::value::Value;
use crate::visibility::Visibility;
use crate::Integer;
use std::collections::HashMap;

use crate::env::Env;

type Scope = HashMap<Symbol, Value>;

pub(super) fn interpret(env: &mut Env, module: &Module) -> Result<Value> {
    Interpreter {
        env,
        module,
        scopes: Default::default(),
    }
    .run()
}

#[derive(Debug)]
struct Interpreter<'a> {
    env: &'a mut Env,
    module: &'a Module,
    scopes: Vec<Scope>,
}

impl<'a> Interpreter<'a> {
    fn set_val(&mut self, symbol: Symbol, value: Value) -> Result<()> {
        if !self.scopes.is_empty() {
            self.scopes.last_mut().unwrap().insert(symbol, value);
            Ok(())
        } else {
            self.env
                .values
                .set(self.module.path.fq_sym(symbol), Visibility::Module, value)
        }
    }

    fn get_val(&self, symbol: &Symbol) -> Option<Value> {
        let mut i = self.scopes.len();
        while i > 0 {
            let result = self.scopes[i - 1].get(symbol);
            if result.is_some() {
                return result.cloned();
            }
            i = i - 1;
        }
        if let Some(vv) = self
            .env
            .values
            .get(&self.module.path.fq_sym(symbol.clone()))
        {
            return Some(vv.unwrap());
        }
        None
    }

    fn run(mut self) -> Result<Value> {
        let mut value = Value::v_none(&Loc::none());
        for e in &self.module.expressions {
            value = self.expression(e)?;
        }
        Ok(value)
    }

    fn expression(&mut self, expr: &Expression) -> Result<Value> {
        match &expr.expr {
            Expr::Value(v) => Ok(v.clone()),
            Expr::Local(l) => match self.get_val(&l.symbol) {
                Some(value) => Ok(value),
                None => expr.rt_err(Error::UnknownLocalSymbol(l.symbol.clone())),
            },
            Expr::Assignment(a) => {
                let value = self.expression(&a.expr)?;
                self.set_val(a.symbol.clone(), value.clone())?;
                Ok(value)
            }
            Expr::Add(t) => self
                .as_two_integers(&t.expr1, &t.expr2)
                .map(|(v1, v2)| Value::integer(&expr.loc, v1 + v2)),
            Expr::Sub(t) => self
                .as_two_integers(&t.expr1, &t.expr2)
                .map(|(v1, v2)| Value::integer(&expr.loc, v1 - v2)),
            Expr::Mul(t) => self
                .as_two_integers(&t.expr1, &t.expr2)
                .map(|(v1, v2)| Value::integer(&expr.loc, v1 * v2)),
            Expr::Div(t) => self.div(&expr.loc, &t.expr1, &t.expr2),
            Expr::LogicalAnd(t) => self.and(&expr.loc, &t.expr1, &t.expr2),
            Expr::LogicalOr(t) => self.or(&expr.loc, &t.expr1, &t.expr2),
            _ => expr.rt_err(Error::NotImplemented),
        }
    }

    fn as_integer(&mut self, expr: &Expression) -> Result<Integer> {
        match self.expression(expr)?.as_integer() {
            Some(v) => Ok(v),
            None => expr.type_mismatch(Type::Integer),
        }
    }

    fn as_two_integers(
        &mut self,
        expr1: &Expression,
        expr2: &Expression,
    ) -> Result<(Integer, Integer)> {
        Error::merge(self.as_integer(expr1), self.as_integer(expr2))
    }

    fn as_boolean(&mut self, expr: &Expression) -> Result<bool> {
        match self.expression(expr)?.as_boolean() {
            Some(v) => Ok(v),
            None => expr.type_mismatch(Type::Boolean),
        }
    }

    fn div(&mut self, loc: &Loc, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        let (v1, v2) = self.as_two_integers(&expr1, &expr2)?;
        // We only have integers for now
        if v2.is_zero() {
            expr2.rt_err(Error::DivisionByZero)
        } else {
            Ok(Value::integer(loc, v1 / v2))
        }
    }

    fn and(&mut self, loc: &Loc, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        if self.as_boolean(expr1)? {
            Ok(Value::boolean(loc, self.as_boolean(expr2)?))
        } else {
            Ok(Value::v_false(loc)) // short-circuit
        }
    }

    fn or(&mut self, loc: &Loc, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        if self.as_boolean(expr1)? {
            Ok(Value::v_true(loc)) // short-circuit
        } else {
            Ok(Value::boolean(loc, self.as_boolean(expr2)?))
        }
    }
}

#[cfg(test)]
mod tests;
