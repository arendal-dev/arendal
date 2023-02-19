use std::rc::Rc;

use super::{lexer, Errors, Expression, Indentation, LexemeKind, LexemeRef, Lexemes, Result};

// Parses a single expression (a single line for now)
pub fn parse_expression(input: &str) -> Result<Expression> {
    let lines = lexer::lex(input)?;
    let lexemes = lines.get_lexemes_at(0).unwrap_or_default();
    Parser::new(lexemes).parse_expression()
}

struct Parser {
    input: Rc<Lexemes>,
    index: usize, // Index of the current input lexer
    errors: Errors,
}

impl Parser {
    fn new(input: Lexemes) -> Parser {
        Parser {
            input: Rc::new(input),
            index: 0,
            errors: Default::default(),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        !self.input.contains(self.index)
    }

    // Consumes one lexer, advancing the index accordingly.
    fn consume(&mut self) {
        self.index += 1;
    }

    // Returns a clone of the lexer at the current index, if any
    fn peek(&self) -> Option<LexemeRef> {
        self.input.get(self.index)
    }

    // Consumes one lexer a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<LexemeRef> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the lexer the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<LexemeRef> {
        self.input.get(self.index + n)
    }

    // Parses the input as single expression.
    fn parse_expression(mut self) -> Result<Expression> {
        if let Some(e) = self.expression() {
            self.errors.to_result(e)
        } else {
            self.empty_input()
        }
    }

    // Parses a single expression, if any, consuming as many tokens as needed.
    // Assumes that the expression starts on a line.
    fn expression(&mut self) -> Option<Expression> {
        if let Some(lexeme) = self.peek() {
            let parser = expr::Parser::new(self.input.clone(), self.index);
            match parser.parse() {
                Ok(maybe) => maybe,
                Err(error) => {
                    self.errors.append(error);
                    // TODO advance until next "resonable" line
                    None
                }
            }
        } else {
            None
        }
    }

    fn add_error(&mut self, lexeme: &LexemeRef, kind: ErrorKind) -> Option<Expression> {
        self.errors.add(Error::new(lexeme, kind));
        None
    }

    fn empty_input<T>(mut self) -> Result<T> {
        self.errors.add(EmptyInputError {});
        Err(self.errors)
    }
}

#[derive(Debug)]
struct EmptyInputError {}

impl super::Error for EmptyInputError {}

#[derive(Debug)]
struct Error {
    lexeme: LexemeRef,
    kind: ErrorKind,
}

impl Error {
    fn new(lexeme: &LexemeRef, kind: ErrorKind) -> Self {
        Error {
            lexeme: lexeme.clone(),
            kind,
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    ParsingError, // placeholder, temporary error
}

impl super::Error for Error {}

mod expr;

#[cfg(test)]
mod tests;
