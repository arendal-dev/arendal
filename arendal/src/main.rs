use twi::ValueResult;

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() -> rustyline::Result<()> {
    let mut rl = Editor::<()>::new()?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => eval_and_print(line.as_str()),
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

fn eval(input: &str) -> ValueResult {
    let parsed = parser::parser::parse_expression(input)?;
    let checked = core::typecheck::expression(parsed)?;
    twi::expression(checked)
}

fn eval_and_print(input: &str) {
    match eval(input) {
        Ok(tv) => {
            println!("{}", tv);
        }
        Err(errors) => {
            println!("{}", errors);
        }
    }
}

#[cfg(test)]
mod tests;
