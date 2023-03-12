use core::ast::Expression;
use core::error::{Error, Errors, Loc, Result};

use crate::lexer::{lex, Lexeme, LexemeKind, Lexemes};

// Parses the input as single expression
pub fn parse_expression(input: &str) -> Result<Expression> {
    let lexemes = lex(input)?;
    Parser::new(lexemes).parse_expression()
}

struct Parser {
    input: Lexemes,
    index: usize, // Index of the current input lexeme
}

impl Parser {
    fn new(input: Lexemes) -> Parser {
        Parser { input, index: 0 }
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
    fn peek(&self) -> Option<Lexeme> {
        self.input.get(self.index)
    }

    // Consumes one line a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<Lexeme> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the line the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<Lexeme> {
        self.input.get(self.index + n)
    }

    // If the next lexeme maches the provided one, advances it and returns true
    fn match1(&mut self, kind: LexemeKind) -> bool {
        if let Some(lexeme) = self.peek() {
            if kind == *lexeme.kind() {
                self.consume();
                return true;
            }
        }
        false
    }

    // Parses the input as a single expression.
    fn parse_expression(mut self) -> Result<Expression> {
        if (self.is_done()) {
            return self.empty_input();
        }
        let expr = expr::parse(&mut self)?;
        if let Some(_) = self.peek() {
            self.expression_expected()
        } else {
            Ok(expr)
        }
    }

    fn err_no_lexeme(&self, error: ParserError) -> Result<Expression> {
        Err(Errors::new(Loc::none(), error))
    }

    fn empty_input(&self) -> Result<Expression> {
        self.err_no_lexeme(ParserError::EmptyInputError)
    }

    fn expression_expected(&self) -> Result<Expression> {
        self.err_no_lexeme(ParserError::ExpressionExpectedError)
    }
}

#[derive(Debug)]
enum ParserError {
    EmptyInputError,
    ExpressionExpectedError,
    ParsingError, // placeholder, temporary error
}

impl Error for ParserError {}

mod expr;

#[cfg(test)]
mod tests;
