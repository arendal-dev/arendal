use super::{Expr, Expression, Module};
use crate::error::{Error, Loc, Result};
use crate::symbol::Symbol;
use crate::value::Value;
use crate::visibility::Visibility;
use crate::Integer;
use std::collections::HashMap;

use crate::env::{Env, RuntimeError};

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
    fn set_val(&mut self, loc: Loc, symbol: Symbol, value: Value) -> Result<()> {
        if !self.scopes.is_empty() {
            self.scopes.last_mut().unwrap().insert(symbol, value);
            return Ok(());
        }
        self.env.values.set(
            loc,
            self.module.path.fq_sym(symbol),
            Visibility::Module,
            value,
        )
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
        let mut value = Value::None;
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
                None => err(expr, RuntimeError::UknownVal(l.symbol.clone())),
            },
            Expr::Assignment(a) => {
                let value = self.expression(&a.expr)?;
                self.set_val(expr.loc.clone(), a.symbol.clone(), value.clone())?;
                Ok(value)
            }
            Expr::Add(t) => self.add(&t.expr1, &t.expr2),
            Expr::Sub(t) => self.sub(&t.expr1, &t.expr2),
            Expr::Mul(t) => self.mul(&t.expr1, &t.expr2),
            Expr::Div(t) => self.div(&t.expr1, &t.expr2),
            _ => err(expr, RuntimeError::NotImplemented),
        }
    }

    fn add(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        let v1 = self.expression(expr1)?;
        let v2 = self.expression(expr2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() + v2.as_integer().unwrap())
    }

    fn sub(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        let v1 = self.expression(expr1)?;
        let v2 = self.expression(expr2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() - v2.as_integer().unwrap())
    }

    fn mul(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        let v1 = self.expression(expr1)?;
        let v2 = self.expression(expr2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() * v2.as_integer().unwrap())
    }

    fn div(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        let v1 = self.expression(expr1)?;
        let v2 = self.expression(expr2)?;
        // We only have integers for now
        let i2 = v2.as_integer().unwrap();
        if i2.is_zero() {
            err(expr2, RuntimeError::DivisionByZero)
        } else {
            integer(v1.as_integer().unwrap() / i2)
        }
    }
}

fn integer(value: Integer) -> Result<Value> {
    Ok(Value::Integer(value))
}

fn err(expr: &Expression, error: RuntimeError) -> Result<Value> {
    Error::err(expr.loc.clone(), error)
}

#[cfg(test)]
mod tests;
