use core::ast::Expression;
use core::error::{Error, Errors, Loc, Result};

use crate::lexer::{lex, Lexeme, Lexemes};

// Parses the input as single expression
pub fn parse_expression(input: &str) -> Result<Expression> {
    let lexemes = lex(input)?;
    Parser::new(lexemes).parse_expression()
}

struct Parser {
    input: Lexemes,
    index: usize, // Index of the current input lexer
    errors: Errors,
}

impl Parser {
    fn new(input: Lexemes) -> Parser {
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

    // Advances the index to point advances by the child parser
    fn advance_child(
        &mut self,
        (result, index): (Result<Expression>, usize),
    ) -> Result<Expression> {
        self.index = index;
        result
    }

    // Parses the input as a single expression.
    fn parse_expression(mut self) -> Result<Expression> {
        let result = self.expression();
        if let Some(_) = self.peek() {
            self.expression_expected()
        } else {
            self.errors.result_to_result(result)
        }
    }

    // Parses an expression in the current position.
    fn expression(&mut self) -> Result<Expression> {
        self.advance_child(expr::Parser::new(self.input.clone(), self.index).parse())
    }

    fn add_error(&mut self, lexeme: &Lexeme, error: ParserError) -> Option<Expression> {
        self.errors.add(lexeme.loc(), error);
        None
    }

    fn err_no_lexeme(mut self, error: ParserError) -> Result<Expression> {
        self.errors.add(Loc::none(), error);
        Err(self.errors)
    }

    fn empty_input(mut self) -> Result<Expression> {
        self.err_no_lexeme(ParserError::EmptyInputError)
    }

    fn expression_expected(mut self) -> Result<Expression> {
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
