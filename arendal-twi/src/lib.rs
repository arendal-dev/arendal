pub mod value;

mod expr;

use core::error::{Error, Result};
use core::identifier::Identifier;
use core::typed::TypedExpr;
use std::collections::HashMap;

use value::Value;

pub type ValueResult = Result<Value>;

#[derive(Debug, Clone, Default)]
struct ValScope {
    vals: HashMap<Identifier, Value>,
}

impl ValScope {
    fn get(&self, id: &Identifier) -> Option<Value> {
        self.vals.get(id).cloned()
    }

    fn set(&mut self, id: Identifier, value: Value) {
        self.vals.insert(id, value);
    }
}

pub struct Interpreter {
    val_scopes: Vec<ValScope>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            val_scopes: vec![Default::default()],
        }
    }

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

    pub fn set_val(&mut self, id: Identifier, value: Value) {
        self.val_scopes.last_mut().unwrap().set(id, value)
    }

    pub fn get_val(&self, id: &Identifier) -> Option<Value> {
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

    pub fn expression(&mut self, expr: &TypedExpr) -> ValueResult {
        expr::eval(self, &expr)
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    UknownVal(Identifier),
    DivisionByZero,
    NotImplemented,
}

impl Error for RuntimeError {}
