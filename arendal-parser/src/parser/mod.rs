use super::{lexer, Errors, Expression, LexemeRef, Lexemes, Line, Lines, Result};

// Parses a single expression (a single line for now)
pub fn parse_expression(input: &str) -> Result<Expression> {
    let lines = lexer::lex(input)?;
    Parser::new(lines).parse_expression()
}

struct Parser {
    input: Lines,
    index: usize, // Index of the current input lexer
    errors: Errors,
}

impl Parser {
    fn new(input: Lines) -> Parser {
        Parser {
            input,
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

    // Returns a clone of the line at the current index, if any
    fn peek(&self) -> Option<Line> {
        self.input.get(self.index)
    }

    // Consumes one line a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<Line> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the line the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<Line> {
        self.input.get(self.index + n)
    }

    // Parses the current line as a single expression.
    fn parse_expression(mut self) -> Result<Expression> {
        if let Some(line) = self.peek() {
            if let Some(e) = self.expression(line.lexemes.clone()) {
                self.errors.to_result(e)
            } else {
                Err(self.errors)
            }
        } else {
            self.empty_input()
        }
    }

    // Parses a list of lexemes as an expression.
    fn expression(&mut self, lexemes: Lexemes) -> Option<Expression> {
        if let Some(lexeme) = self.peek() {
            let parser = expr::Parser::new(lexemes);
            match parser.parse() {
                Ok(maybe) => maybe,
                Err(error) => {
                    self.errors.append(error);
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
