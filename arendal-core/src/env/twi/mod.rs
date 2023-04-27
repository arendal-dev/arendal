use crate::ast::BinaryOp;
use crate::error::{Error, Errors, Loc, Result};
use crate::symbol::Symbol;
use crate::typed::{Expr, Expression, Module};
use crate::value::Value;
use crate::visibility::Visibility;
use crate::Integer;
use std::collections::HashMap;

use super::Env;

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
        match expr.borrow_expr() {
            Expr::Value(v) => Ok(v.clone()),
            Expr::LocalSymbol(id) => match self.get_val(id) {
                Some(value) => Ok(value),
                None => err(expr, RuntimeError::UknownVal(id.clone())),
            },
            Expr::Assignment(id, expr) => {
                let value = self.expression(expr)?;
                self.set_val(expr.clone_loc(), id.clone(), value.clone())?;
                Ok(value)
            }
            Expr::Binary(op, e1, e2) => self.binary(*op, e1, e2),
            _ => err(expr, RuntimeError::NotImplemented),
        }
    }

    fn binary(&mut self, op: BinaryOp, e1: &Expression, e2: &Expression) -> Result<Value> {
        let v1 = self.expression(e1)?;
        match op {
            BinaryOp::Add => self.add(v1, e2),
            BinaryOp::Sub => self.sub(v1, e2),
            BinaryOp::Mul => self.mul(v1, e2),
            BinaryOp::Div => self.div(v1, e2),
            _ => err(e1, RuntimeError::NotImplemented),
        }
    }

    fn add(&mut self, v1: Value, e2: &Expression) -> Result<Value> {
        let v2 = self.expression(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() + v2.as_integer().unwrap())
    }

    fn sub(&mut self, v1: Value, e2: &Expression) -> Result<Value> {
        let v2 = self.expression(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() - v2.as_integer().unwrap())
    }

    fn mul(&mut self, v1: Value, e2: &Expression) -> Result<Value> {
        let v2 = self.expression(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() * v2.as_integer().unwrap())
    }

    fn div(&mut self, v1: Value, e2: &Expression) -> Result<Value> {
        let v2 = self.expression(e2)?;
        // We only have integers for now
        let i2 = v2.as_integer().unwrap();
        if i2.is_zero() {
            err(e2, RuntimeError::DivisionByZero)
        } else {
            integer(v1.as_integer().unwrap() / i2)
        }
    }
}

fn integer(value: Integer) -> Result<Value> {
    Ok(Value::Integer(value))
}

fn err(expr: &Expression, error: RuntimeError) -> Result<Value> {
    Errors::err(expr.clone_loc(), error)
}

#[derive(Debug)]
pub enum RuntimeError {
    UknownVal(Symbol),
    DivisionByZero,
    NotImplemented,
}

impl Error for RuntimeError {}

#[cfg(test)]
mod tests;
