pub mod pos;
pub mod tokenizer1; // Tokenizer - first pass
pub mod tokenizer2; // Tokenizer - second pass

use arendal_ast::Expression;
use arendal_ast::error::{Error, Errors, Result};
pub use pos::Pos;
use tokenizer2::{Token, Tokens};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indentation {
    tabs: usize,
    spaces: usize,
}

impl Indentation {
    #[inline]
    fn new(tabs: usize, spaces: usize) -> Indentation {
        Indentation { tabs, spaces }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewLine {
    LF,
    CRLF,
}

impl NewLine {
    fn bytes(self) -> usize {
        match self {
            Self::LF => 1,
            Self::CRLF => 2,
        }
    }

    fn chars(self) -> usize {
        self.bytes() // we have another method in case it's different in the future
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enclosure {
    Parens,
    Square,
    Curly,
}

#[derive(Debug)]
struct ParserError<'a> {
    // Error position
    pos: Pos<'a>,
    error_type: ErrorType<'a>,
}

impl<'a> ParserError<'a> {
    fn new(pos: Pos<'a>, error_type: ErrorType<'a>) -> Self {
        ParserError { pos, error_type }
    }
}

#[derive(Debug)]
enum ErrorType<'a> {
    IndentationError,
    UnexpectedChar(char),
    UnexpectedToken(tokenizer1::Token<'a>),
    TBDError(Token<'a>), // placeholder, temporary error
}

impl<'a> Error for ParserError<'a> {}

fn indentation_error(pos: Pos) -> ParserError {
    ParserError::new(pos, ErrorType::IndentationError)
}

fn unexpected_char(pos: Pos, c: char) -> ParserError {
    ParserError::new(pos, ErrorType::UnexpectedChar(c))
}

fn unexpected_token(token: tokenizer1::Token) -> ParserError {
    ParserError::new(token.pos, ErrorType::UnexpectedToken(token))
}

fn tbd_error(token: Token) -> ParserError {
    ParserError::new(token.pos, ErrorType::TBDError(token))
}

struct Parser<'a> {
    input: Tokens<'a>,
    index: usize, // Index of the current input token
}

impl<'a> Parser<'a> {
    fn new(input: Tokens<'a>) -> Parser<'a> {
        Parser { input, index: 0 }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.index >= self.input.len()
    }

    // Consumes one token, advancing the index accordingly.
    fn consume(&mut self) {
        self.index += 1;
    }

    // Returns a clone of the token at the current index, if any
    fn peek(&self) -> Option<Box<Token<'a>>> {
        if self.is_done() {
            None
        } else {
            Some(self.input[self.index].clone())
        }
    }

    // Consumes one token a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<Box<Token<'a>>> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the token the requested positions after the current one, if any.
    fn peek_other(&self, n: usize) -> Option<Box<Token<'a>>> {
        let i = self.index + n;
        if i >= self.input.len() {
            None
        } else {
            Some(self.input[i].clone())
        }
    }

    // Tries to parses an expression, if any, consuming as many tokens as needed
    fn expression(&mut self) -> Option<Expression> {
        None
    }

}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
