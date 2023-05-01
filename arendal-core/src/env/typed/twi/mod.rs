use super::{Expr, Expression, Module};
use crate::error::{Loc, Result};
use crate::symbol::Symbol;
use crate::types::Type;
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
    fn set_val(&mut self, loc: &Loc, symbol: Symbol, value: Value) -> Result<()> {
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
                None => expr.rt_err(RuntimeError::UknownVal(l.symbol.clone())),
            },
            Expr::Assignment(a) => {
                let value = self.expression(&a.expr)?;
                self.set_val(&expr.loc, a.symbol.clone(), value.clone())?;
                Ok(value)
            }
            Expr::Add(t) => self.add(&t.expr1, &t.expr2),
            Expr::Sub(t) => self.sub(&t.expr1, &t.expr2),
            Expr::Mul(t) => self.mul(&t.expr1, &t.expr2),
            Expr::Div(t) => self.div(&t.expr1, &t.expr2),
            Expr::LogicalAnd(t) => self.and(&t.expr1, &t.expr2),
            Expr::LogicalOr(t) => self.or(&t.expr1, &t.expr2),
            _ => expr.rt_err(RuntimeError::NotImplemented),
        }
    }

    fn as_integer(&mut self, expr: &Expression) -> Result<Integer> {
        match self.expression(expr)?.as_integer() {
            Some(v) => Ok(v),
            None => expr.type_mismatch(Type::Integer),
        }
    }

    fn as_boolean(&mut self, expr: &Expression) -> Result<bool> {
        match self.expression(expr)?.as_boolean() {
            Some(v) => Ok(v),
            None => expr.type_mismatch(Type::Boolean),
        }
    }

    fn add(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        let v1 = self.as_integer(expr1)?;
        let v2 = self.as_integer(expr2)?;
        // We only have integers for now
        integer(v1 + v2)
    }

    fn sub(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        let v1 = self.as_integer(expr1)?;
        let v2 = self.as_integer(expr2)?;
        // We only have integers for now
        integer(v1 - v2)
    }

    fn mul(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        let v1 = self.as_integer(expr1)?;
        let v2 = self.as_integer(expr2)?;
        // We only have integers for now
        integer(v1 * v2)
    }

    fn div(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        let v1 = self.as_integer(expr1)?;
        let v2 = self.as_integer(expr2)?;
        // We only have integers for now
        if v2.is_zero() {
            expr2.rt_err(RuntimeError::DivisionByZero)
        } else {
            integer(v1 / v2)
        }
    }

    fn and(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        if self.as_boolean(expr1)? {
            Ok(Value::boolean(self.as_boolean(expr2)?))
        } else {
            Ok(Value::False) // short-circuit
        }
    }

    fn or(&mut self, expr1: &Expression, expr2: &Expression) -> Result<Value> {
        if self.as_boolean(expr1)? {
            Ok(Value::True) // short-circuit
        } else {
            Ok(Value::boolean(self.as_boolean(expr2)?))
        }
    }
}

fn integer(value: Integer) -> Result<Value> {
    Ok(Value::Integer(value))
}

#[cfg(test)]
mod tests;
