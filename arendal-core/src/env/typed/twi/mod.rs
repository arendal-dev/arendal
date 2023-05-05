use super::{Expr, Expression, Expressions, Module, TwoInts};
use crate::error::{Error, Loc, Result};
use crate::symbol::Symbol;
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
        self.expressions(&Loc::none(), &self.module.expressions)
    }

    fn expressions(&mut self, loc: &Loc, exprs: &Expressions) -> Result<Value> {
        let mut value = Value::v_none(&Loc::none());
        for e in exprs {
            value = self.expression(e)?;
        }
        Ok(value)
    }

    fn expression(&mut self, expr: &Expression) -> Result<Value> {
        match &expr.expr {
            Expr::Value(v) => Ok(v.clone()),
            Expr::Local(l) => match self.get_val(&l.symbol) {
                Some(value) => Ok(value),
                None => expr.err(Error::UnknownLocalSymbol(l.symbol.clone())),
            },
            Expr::Conditional(c) => {
                if self.expression(&c.expr)?.as_boolean()? {
                    self.expression(&c.then)
                } else {
                    self.expression(&c.otherwise)
                }
            }
            Expr::Assignment(a) => {
                let value = self.expression(&a.expr)?;
                self.set_val(a.symbol.clone(), value.clone())?;
                Ok(value)
            }
            Expr::IntAdd(t) => self
                .eval_two_ints(t)
                .map(|(v1, v2)| Value::integer(&expr.loc, v1 + v2)),
            Expr::IntSub(t) => self
                .eval_two_ints(t)
                .map(|(v1, v2)| Value::integer(&expr.loc, v1 - v2)),
            Expr::IntMul(t) => self
                .eval_two_ints(t)
                .map(|(v1, v2)| Value::integer(&expr.loc, v1 * v2)),
            Expr::IntDiv(t) => self.div(&expr.loc, t),
            Expr::LogicalAnd(t) => self.and(&expr.loc, &t.expr1, &t.expr2),
            Expr::LogicalOr(t) => self.or(&expr.loc, &t.expr1, &t.expr2),
            Expr::Block(exprs) => {
                self.scopes.push(Scope::default());
                let value = self.expressions(&expr.loc, exprs);
                self.scopes.pop();
                value
            }
            _ => expr.err(Error::NotImplemented),
        }
    }

    fn eval_two_ints(&mut self, t: &TwoInts) -> Result<(Integer, Integer)> {
        Error::merge(
            self.expression(&t.expr1)?.as_integer(),
            self.expression(&t.expr2)?.as_integer(),
        )
    }

    fn eval_bool(&mut self, expr: &Expression) -> Result<bool> {
        self.expression(expr)?.as_boolean()
    }

    fn div(&mut self, loc: &Loc, t: &TwoInts) -> Result<Value> {
        let (v1, v2) = self.eval_two_ints(t)?;
        // We only have integers for now
        if v2.is_zero() {
            loc.err(Error::DivisionByZero)
        } else {
            Ok(Value::integer(loc, v1 / v2))
        }
    }

    fn and(&mut self, loc: &Loc, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        if self.eval_bool(expr1)? {
            Ok(Value::boolean(loc, self.eval_bool(expr2)?))
        } else {
            Ok(Value::v_false(loc)) // short-circuit
        }
    }

    fn or(&mut self, loc: &Loc, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        if self.eval_bool(expr1)? {
            Ok(Value::v_true(loc)) // short-circuit
        } else {
            Ok(Value::boolean(loc, self.eval_bool(expr2)?))
        }
    }
}

#[cfg(test)]
mod tests;
