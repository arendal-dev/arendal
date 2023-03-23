use core::env::{EnvRef, Module};
use twi::{Interpreter, ValueResult};

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() -> rustyline::Result<()> {
    let mut repl = REPL::new();
    repl.run()
}

struct REPL {
    module: Module,
    interpreter: Interpreter,
}

impl REPL {
    fn new() -> Self {
        REPL {
            module: EnvRef::new_with_prelude().empty_local_module().unwrap(),
            interpreter: Interpreter::new(),
        }
    }
    fn run(&mut self) -> rustyline::Result<()> {
        let mut rl = Editor::<()>::new()?;
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => self.eval_and_print(line.as_str()),
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        Ok(())
    }

    fn eval(&mut self, input: &str) -> ValueResult {
        let parsed = parser::parser::parse_expression(input)?;
        let checked = self.module.expression(&parsed)?;
        self.interpreter.expression(&checked)
    }

    fn eval_and_print(&mut self, input: &str) {
        match self.eval(input) {
            Ok(tv) => {
                println!("{}", tv);
            }
            Err(errors) => {
                println!("{}", errors);
            }
        }
    }
}

#[cfg(test)]
mod tests;
