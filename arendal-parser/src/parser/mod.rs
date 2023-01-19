use super::{error, lexer, Errors, Expression, LexemeKind, LexemeRef, Lexemes, Result};
use std::rc::Rc;

// Tries to parses an expression
fn parse_expression(input: &str) -> Result<Option<Expression>> {
    let pass2 = lexer::lex(input)?;
    Parser::new(pass2).expression()
}

struct Parser<'a> {
    input: Lexemes<'a>,
    index: usize, // Index of the current input lexer
    errors: Errors<'a>,
}

impl<'a> Parser<'a> {
    fn new(input: Lexemes<'a>) -> Parser<'a> {
        Parser {
            input,
            index: 0,
            errors: Default::default(),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.input.contains(self.index)
    }

    // Consumes one lexer, advancing the index accordingly.
    fn consume(&mut self) {
        self.index += 1;
    }

    // Returns a clone of the lexer at the current index, if any
    fn peek(&self) -> Option<LexemeRef<'a>> {
        self.input.get(self.index)
    }

    // Consumes one lexer a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<LexemeRef<'a>> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the lexer the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<LexemeRef<'a>> {
        self.input.get(self.index + n)
    }

    // Tries to parses an expression, if any, consuming as many tokens as needed
    fn expression(mut self) -> Result<'a, Option<Expression<'a>>> {
        Ok(None)
    }

    fn rule_expression(&mut self) -> Option<Expression<'a>> {
        self.rule_primary()
    }

    fn rule_primary(&mut self) -> Option<Expression<'a>> {
        let lexeme = self.peek()?;
        match &lexeme.kind {
            LexemeKind::Integer(n) => {
                self.consume();
                Some(Expression::int_literal(lexeme.pos, n.clone()))
            }
            _ => {
                self.add_error(&lexeme, ErrorKind::ParsingError);
                None
            }
        }
    }

    fn add_error(&mut self, lexeme: &LexemeRef<'a>, kind: ErrorKind) {
        self.errors.add(Error::new(Rc::clone(lexeme), kind))
    }
}

#[derive(Debug)]
struct Error<'a> {
    lexeme: LexemeRef<'a>,
    kind: ErrorKind,
}

impl<'a> Error<'a> {
    fn new(lexeme: LexemeRef<'a>, kind: ErrorKind) -> Self {
        Error { lexeme, kind }
    }
}

#[derive(Debug)]
enum ErrorKind {
    ParsingError, // placeholder, temporary error
}

impl<'a> error::Error<'a> for Error<'a> {}

#[cfg(test)]
mod tests;
