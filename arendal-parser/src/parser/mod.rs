use super::{lexer, Lexeme, LexemeKind, Lexemes};
use crate::{Errors, Expression, Result};

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
        self.index >= self.input.len()
    }

    // Consumes one lexer, advancing the index accordingly.
    fn consume(&mut self) {
        self.index += 1;
    }

    // Returns a clone of the lexer at the current index, if any
    fn peek(&self) -> Option<Box<Lexeme<'a>>> {
        if self.is_done() {
            None
        } else {
            Some(self.input[self.index].clone())
        }
    }

    // Consumes one lexer a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<Box<Lexeme<'a>>> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the lexer the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<Box<Lexeme<'a>>> {
        let i = self.index + n;
        if i >= self.input.len() {
            None
        } else {
            Some(self.input[i].clone())
        }
    }

    // Tries to parses an expression, if any, consuming as many tokens as needed
    fn expression(mut self) -> Result<'a, Option<Expression<'a>>> {
        Ok(None)
    }

    fn rule_expression(&mut self) -> Option<Expression<'a>> {
        self.rule_primary()
    }

    fn rule_primary(&mut self) -> Option<Expression<'a>> {
        let token = self.peek()?;
        match token.kind {
            LexemeKind::Integer(n) => {
                self.consume();
                Some(Expression::int_literal(token.pos, n))
            }
            _ => {
                self.errors.add(crate::parsing_error(token.pos));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests;
