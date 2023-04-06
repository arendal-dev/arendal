use crate::ast::BinaryOp;
use crate::error::{Error, Errors, Result};
use crate::symbol::Symbol;
use crate::typed::{TExpr, TypedExpr};
use crate::value::Value;
use crate::Integer;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
struct ValScope {
    vals: HashMap<Symbol, Value>,
}

impl ValScope {
    fn get(&self, id: &Symbol) -> Option<Value> {
        self.vals.get(id).cloned()
    }

    fn set(&mut self, id: Symbol, value: Value) {
        self.vals.insert(id, value);
    }
}

#[derive(Default)]
pub(super) struct Interpreter {
    val_scopes: Vec<ValScope>,
}

impl Interpreter {
    pub fn push_val_scope(&mut self) -> usize {
        self.val_scopes.push(Default::default());
        self.val_scopes.len()
    }

    pub fn pop_val_scope(&mut self, key: usize) {
        assert!(
            key > 1 && key == self.val_scopes.len(),
            "Removing wrong val scope"
        );
        self.val_scopes.pop();
    }

    pub fn set_val(&mut self, id: Symbol, value: Value) {
        self.val_scopes.last_mut().unwrap().set(id, value)
    }

    pub fn get_val(&self, id: &Symbol) -> Option<Value> {
        let mut i = self.val_scopes.len();
        while i > 0 {
            let result = self.val_scopes[i - 1].get(id);
            if result.is_some() {
                return result;
            }
            i = i - 1;
        }
        None
    }

    pub fn expression(&mut self, expr: &TypedExpr) -> Result<Value> {
        match expr.borrow_expr() {
            TExpr::Value(v) => Ok(v.clone()),
            TExpr::LocalSymbol(id) => match self.get_val(id) {
                Some(value) => Ok(value),
                None => err(expr, RuntimeError::UknownVal(id.clone())),
            },
            TExpr::Assignment(id, expr) => {
                let value = self.expression(expr)?;
                self.set_val(id.clone(), value.clone());
                Ok(value)
            }
            TExpr::Binary(op, e1, e2) => self.binary(*op, e1, e2),
            _ => err(expr, RuntimeError::NotImplemented),
        }
    }

    fn binary(&mut self, op: BinaryOp, e1: &TypedExpr, e2: &TypedExpr) -> Result<Value> {
        let v1 = self.expression(e1)?;
        match op {
            BinaryOp::Add => self.add(v1, e2),
            BinaryOp::Sub => self.sub(v1, e2),
            BinaryOp::Mul => self.mul(v1, e2),
            BinaryOp::Div => self.div(v1, e2),
            _ => err(e1, RuntimeError::NotImplemented),
        }
    }

    fn add(&mut self, v1: Value, e2: &TypedExpr) -> Result<Value> {
        let v2 = self.expression(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() + v2.as_integer().unwrap())
    }

    fn sub(&mut self, v1: Value, e2: &TypedExpr) -> Result<Value> {
        let v2 = self.expression(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() - v2.as_integer().unwrap())
    }

    fn mul(&mut self, v1: Value, e2: &TypedExpr) -> Result<Value> {
        let v2 = self.expression(e2)?;
        // We only have integers for now
        integer(v1.as_integer().unwrap() * v2.as_integer().unwrap())
    }

    fn div(&mut self, v1: Value, e2: &TypedExpr) -> Result<Value> {
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

fn err(expr: &TypedExpr, error: RuntimeError) -> Result<Value> {
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
