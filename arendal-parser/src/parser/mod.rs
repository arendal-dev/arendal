use super::{lexer, Errors, Expression, LexemeRef, Lexemes, Loc, Result};

// Parses a single expression (a single line for now)
pub fn parse_expression(input: &str) -> Result<Expression> {
    let lexemes = lexer::lex(input)?;
    println!("{:?}", lexemes);
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
    fn peek(&self) -> Option<LexemeRef> {
        self.input.get(self.index)
    }

    // Consumes one line a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<LexemeRef> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the line the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<LexemeRef> {
        self.input.get(self.index + n)
    }

    // Parses a single expression.
    fn parse_expression(mut self) -> Result<Expression> {
        if let Some(line) = self.peek() {
            self.errors
                .result_to_result(expr::Parser::new(self.input).parse())
        } else {
            self.expression_expected()
        }
    }

    fn add_error(&mut self, lexeme: &LexemeRef, error: Error) -> Option<Expression> {
        self.errors.add(lexeme.pos().into(), error);
        None
    }

    fn err_no_lexeme<T: super::Error + 'static>(mut self, error: T) -> Result<Expression> {
        self.errors.add(Loc::none(), error);
        Err(self.errors)
    }

    fn empty_input(mut self) -> Result<Expression> {
        self.err_no_lexeme(EmptyInputError {})
    }

    fn expression_expected(mut self) -> Result<Expression> {
        self.err_no_lexeme(ExpressionExpectedError {})
    }
}

#[derive(Debug)]
struct EmptyInputError {}

impl super::Error for EmptyInputError {}

#[derive(Debug)]
struct ExpressionExpectedError {}

impl super::Error for ExpressionExpectedError {}

#[derive(Debug)]
enum Error {
    ParsingError, // placeholder, temporary error
}

impl super::Error for Error {}

mod expr;

#[cfg(test)]
mod tests;
