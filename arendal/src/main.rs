use core::env::Interactive;
use core::error::Result;
use core::value::Value;

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() -> rustyline::Result<()> {
    let mut repl = REPL::new();
    repl.run()
}

struct REPL {
    interactive: Interactive,
}

impl REPL {
    fn new() -> Self {
        REPL {
            interactive: Interactive::default(),
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

    fn eval(&mut self, input: &str) -> Result<Value> {
        self.interactive.run(input)
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
