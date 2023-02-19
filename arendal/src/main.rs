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
    let parsed = parser::parse_expression(input)?;
    if let Some(expr) = parsed {
        let checked = typecheck::expression(expr)?;
        twi::expression(checked)
    } else {
        Ok(twi::Value::Integer(0.into()))
    }
}

fn eval_and_print(input: &str) {
    match eval(input) {
        Ok(tv) => {
            println!("{}", tv);
        }
        Err(_) => {
            println!("Error");
        }
    }
}

#[cfg(test)]
mod tests;
