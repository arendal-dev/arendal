use super::tokenizer1::Token as Token1;
use super::tokenizer1::TokenType as TokenType1;
use super::tokenizer1::Tokens as Tokens1;
use super::{Indentation, NewLine, Pos};
use arendal_error::{Errors, Result};

pub fn tokenize<'a>(input: &'a Tokens1<'a>) -> Result<Tokens<'a>> {
    Tokenizer::new(input).tokenize()
}

#[derive(Debug)]
pub struct Tokens<'a> {
    input: &'a str,
    tokens: Vec<Token<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pos: Pos, // Starting position of the token
    token_type: TokenType<'a>,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenType<'a> {
    Indent(Indentation),
    Spaces(usize),
    Tabs(usize),
    EndOfLine(NewLine),
    Plus,
    Minus,
    Star,
    Slash,
    Dot,
    Greater,
    Less,
    Bang,
    Equal,
    OpenParens,
    CloseParens,
    OpenCBracket,
    CloseCBracket,
    OpenSBracket,
    CloseSBracket,
    Underscore,
    Digits(&'a str),
    Word(&'a str),
}

impl<'a> TokenType<'a> {
    fn single(c: char) -> Option<TokenType<'a>> {
        match c {
            '+' => Some(TokenType::Plus),
            '-' => Some(TokenType::Minus),
            '*' => Some(TokenType::Star),
            '/' => Some(TokenType::Slash),
            '.' => Some(TokenType::Dot),
            '>' => Some(TokenType::Greater),
            '<' => Some(TokenType::Less),
            '!' => Some(TokenType::Bang),
            '=' => Some(TokenType::Equal),
            '(' => Some(TokenType::OpenParens),
            ')' => Some(TokenType::CloseParens),
            '{' => Some(TokenType::OpenCBracket),
            '}' => Some(TokenType::CloseCBracket),
            '[' => Some(TokenType::OpenSBracket),
            ']' => Some(TokenType::CloseSBracket),
            '_' => Some(TokenType::Underscore),
            _ => None,
        }
    }
}

struct Tokenizer<'a> {
    input: &'a Tokens1<'a>,
    tokens: Vec<Token<'a>>,
    errors: Errors,
    input_index: usize, // Index of the current input token
    token_start: Pos,   // Start of the current output token
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a Tokens1) -> Tokenizer<'a> {
        Tokenizer {
            input,
            tokens: Vec::new(),
            errors: Errors::new(),
            input_index: 0,
            token_start: Pos::new(),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.input_index >= self.input.tokens.len()
    }

    // Consumes one token, advancing the index accordingly.
    fn consume(&mut self) {
        self.input_index += 1;
    }

    // Returns a clone of the token at the current index, if any
    fn peek(&self) -> Option<Token1<'a>> {
        if self.is_done() {
            None
        } else {
            Some(self.input.tokens[self.input_index].clone())
        }
    }

    // Consumes one token a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<Token1<'a>> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the token the requested positions after the current one, if any.
    fn peek_other(&self, n: usize) -> Option<Token1<'a>> {
        let i = self.input_index + n;
        if i >= self.input.tokens.len() {
            None
        } else {
            Some(self.input.tokens[i].clone())
        }
    }

    fn tokenize(mut self) -> Result<Tokens<'a>> {
        let mut last_seen_line = 0;
        while let Some(t) = self.peek() {
            self.token_start = t.pos;
            if t.pos.line > last_seen_line {
                last_seen_line = t.pos.line;
                self.consume_indentation(t);
                continue;
            }
            match t {
                _ => self.errors.add(super::unexpected_token()),
            }
        }
        self.errors.to_result(Tokens {
            input: self.input.input,
            tokens: self.tokens,
        })
    }

    fn add_token(&mut self, token_type: TokenType<'a>) {
        self.tokens.push(Token {
            pos: self.token_start,
            token_type,
        });
    }

    fn consume_tabs(&mut self, mut t: TokenType1<'a>) -> usize {
        let mut tabs = 0;
        while let TokenType1::Tabs(n) = t {
            tabs += n;
            if let Some(next) = self.consume_and_peek() {
                t = next.token_type;
            } else {
                break;
            }
        }
        tabs
    }

    fn consume_spaces(&mut self, mut t: TokenType1<'a>) -> usize {
        let mut spaces = 0;
        while let TokenType1::Spaces(n) = t {
            spaces += n;
            if let Some(next) = self.consume_and_peek() {
                t = next.token_type;
            } else {
                break;
            }
        }
        spaces
    }

    fn skip_whitespace(&mut self) {
        while let Some(t) = self.peek() {
            if t.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    fn consume_indentation(&mut self, token: Token1<'a>) {
        let tabs = self.consume_tabs(token.clone().token_type);
        let spaces = self.consume_spaces(token.token_type);
        self.add_token(TokenType::Indent(Indentation::new(tabs, spaces)));
        if let Some(t) = self.peek() {
            if t.is_whitespace() {
                self.add_indentation_error(&t);
                self.skip_whitespace();
            }
        }
    }

    fn add_indentation_error(&mut self, token: &Token1) {
        self.errors.add(super::indentation_error(token.pos))
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, TokenType};
    use super::{Token1, TokenType1};
    use crate::{NewLine, Pos};
    use NewLine::*;

    struct TestCase<'a> {
        pos: Pos,
        tokens: Vec<Token<'a>>,
    }

    impl<'a> TestCase<'a> {
        fn new() -> TestCase<'a> {
            TestCase {
                pos: Pos::new(),
                tokens: Vec::new(),
            }
        }

        fn token(mut self, token_type: TokenType<'a>) -> Self {
            self.tokens.push(Token {
                pos: self.pos,
                token_type,
            });
            self
        }
    }

    #[test]
    fn empty() {
        // TestCase::new().ok("");
    }
}
