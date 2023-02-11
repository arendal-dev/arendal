use ast::error::Result;
use wti::TypedValue;

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

fn eval(input: &str) -> Result<TypedValue> {
    let parsed = parser::parse_expression(input)?;
    let checked = typecheck::expression(&parsed)?;
    wti::expression(&checked)
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
