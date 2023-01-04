mod errors;

use arendal_error::{ErrorCollector, Result};
use super::Indentation;
use errors::*;

pub fn scan(input: &str) -> Result<Vec<Token>> {
    let mut scanner = Scanner {
        input,
        tokens: Vec::new(),
        errors: ErrorCollector::new(),
        line: 0,
    };
    scanner.scan();
    scanner.errors.to_result(scanner.tokens)
}



#[derive(Debug)]
pub struct Token;

struct Scanner<'a> {
    input : &'a str,
    // TODO: add chars with index
    tokens : Vec<Token>,
    errors : ErrorCollector,
    line : usize,
}

impl<'a> Scanner<'a> {
    fn scan(&mut self) {
    }

    fn start_line(&mut self) {
        // TODO: check we are not done
        self.line = self.line + 1;
        let (indentation, ok) = Indentation::get(self.input); // TODO change for start of the line.
        if !ok {
            self.errors.add(indentation_error())
        }

    }

}

