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

    // Consumes n lexemes, advancing the index accordingly.
    #[inline]
    fn consume_n(&mut self, n: usize) {
        self.index += n;
    }

    // Consumes one lexeme, advancing the index accordingly.
    #[inline]
    fn consume(&mut self) {
        self.consume_n(1);
    }

    // Returns the lexeme at the current index, if any
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

    // Returns the lexeme at the current index, if it matches the provided kind.
    fn peek_if_match(&self, kind: LexemeKind) -> Option<Lexeme> {
        if let Some(lexeme) = self.peek() {
            if kind == *lexeme.kind() {
                return Some(lexeme);
            }
        }
        None
    }

    // Consumes and returns the lexeme at the current index, if it matches the provided kind.
    fn consume_if_match(&mut self, kind: LexemeKind) -> Option<Lexeme> {
        if let Some(lexeme) = self.peek_if_match(kind) {
            self.consume();
            Some(lexeme)
        } else {
            None
        }
    }

    // If the next lexeme maches the provided one, advances it and returns true
    fn match1(&mut self, kind: LexemeKind) -> bool {
        self.consume_if_match(kind).is_some()
    }

    // Parses the input as a single expression.
    fn parse_expression(mut self) -> Result<Expression> {
        if self.is_done() {
            return self.empty_input();
        }
        let expr = expr::parse(&mut self)?;
        if let Some(_) = self.peek() {
            self.expression_expected()
        } else {
            Ok(expr)
        }
    }

    fn ok_n<T>(&mut self, n: usize, value: T) -> Result<T> {
        self.consume_n(n);
        Ok(value)
    }

    #[inline]
    fn ok<T>(&mut self, value: T) -> Result<T> {
        self.ok_n(1, value)
    }

    fn err<T>(&self, error: ParserError) -> Result<T> {
        // TODO: try end of previous lexeme
        let loc = if let Some(lexeme) = self.peek() {
            lexeme.loc()
        } else {
            Loc::none()
        };
        Err(Errors::new(loc, error))
    }

    fn err_no_lexeme(&self, error: ParserError) -> Result<Expression> {
        Err(Errors::new(Loc::none(), error))
    }

    fn empty_input(&self) -> Result<Expression> {
        self.err_no_lexeme(ParserError::EmptyInputError)
    }

    fn expression_expected(&self) -> Result<Expression> {
        self.err(ParserError::ExpressionExpectedError)
    }
}

#[derive(Debug)]
enum ParserError {
    EmptyInputError,
    ExpressionExpectedError,
    LValueExpectedError,
    AssignmentExpended,
    ParsingError, // placeholder, temporary error
}

impl Error for ParserError {}

mod expr;

#[cfg(test)]
mod tests;
