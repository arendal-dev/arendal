mod tst;

use crate::{
    error::Result,
    types::Types,
    values::{Value, Values},
};

#[derive(Debug, Clone, Default)]
pub struct Env {
    types: Types,
    values: Values,
}

impl Env {
    pub fn run(&mut self, input: &str) -> Result<Value> {
        tst::run(self, input)
    }
}
