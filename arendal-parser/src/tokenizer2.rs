use super::tokenizer1::Token as Token1;
use super::tokenizer1::TokenType as TokenType1;
use super::tokenizer1::Tokens as Tokens1;
use super::{Errors, Indentation, NewLine, Pos, Result};
use num::bigint::{BigInt, ToBigInt};

pub fn tokenize(input: &str) -> Result<Tokens> {
    let pass1 = super::tokenizer1::tokenize(input)?;
    tokenize2(pass1)
}

fn tokenize2(input: Tokens1) -> Result<Tokens> {
    Tokenizer::new(input).tokenize()
}

type TokenVec<'a> = Vec<Box<Token<'a>>>;

#[derive(Debug)]
pub struct Tokens<'a> {
    input: &'a str,
    tokens: TokenVec<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pos: Pos, // Starting position of the token
    token_type: TokenType<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenType<'a> {
    Indent(Indentation),
    Whitespace,
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
    NotEqual,
    Integer(BigInt),
    OpenParens,
    CloseParens,
    OpenCBracket,
    CloseCBracket,
    OpenSBracket,
    CloseSBracket,
    Underscore,
    Word(&'a str),
}

impl<'a> TokenType<'a> {
    fn single(t: &TokenType1) -> Option<TokenType<'a>> {
        match t {
            TokenType1::Plus => Some(TokenType::Plus),
            TokenType1::Minus => Some(TokenType::Minus),
            TokenType1::Star => Some(TokenType::Star),
            TokenType1::Slash => Some(TokenType::Slash),
            TokenType1::Dot => Some(TokenType::Dot),
            TokenType1::Greater => Some(TokenType::Greater),
            TokenType1::Less => Some(TokenType::Less),
            TokenType1::Equal => Some(TokenType::Equal),
            TokenType1::Underscore => Some(TokenType::Underscore),
            _ => None,
        }
    }
}

struct Tokenizer<'a> {
    input: Tokens1<'a>,
    tokens: TokenVec<'a>,
    errors: Errors,
    input_index: usize, // Index of the current input token
    token_start: Pos,   // Start of the current output token
}

impl<'a> Tokenizer<'a> {
    fn new(input: Tokens1<'a>) -> Tokenizer<'a> {
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
            if self.consume_whitespace(true) {
                continue;
            }
            if let Some(tt) = TokenType::single(&t.token_type) {
                self.consume();
                self.add_token(tt);
                continue;
            }
            match t.token_type {
                TokenType1::EndOfLine(_) => self.consume(),
                TokenType1::Bang => self.consume_bang(),
                TokenType1::Digits(s) => self.consume_digits(s),
                _ => self.errors.add(super::unexpected_token()),
            }
        }
        self.errors.to_result(Tokens {
            input: self.input.input,
            tokens: self.tokens,
        })
    }

    fn add_token(&mut self, token_type: TokenType<'a>) {
        self.tokens.push(Box::new(Token {
            pos: self.token_start,
            token_type,
        }));
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

    fn consume_whitespace(&mut self, add_token: bool) -> bool {
        let mut found = false;
        while let Some(t) = self.peek() {
            if t.is_whitespace() {
                found = true;
                self.consume();
            } else {
                break;
            }
        }
        if found && add_token {
            self.add_token(TokenType::Whitespace);
        }
        found
    }

    fn consume_indentation(&mut self, token: Token1<'a>) {
        let tabs = self.consume_tabs(token.clone().token_type);
        let spaces = self.consume_spaces(token.token_type);
        self.add_token(TokenType::Indent(Indentation::new(tabs, spaces)));
        if let Some(t) = self.peek() {
            if t.is_whitespace() {
                self.add_indentation_error(&t);
                self.consume_whitespace(false);
            }
        }
    }

    fn consume_bang(&mut self) {
        let t = if let Some(TokenType1::Equal) = self.peek_other(1).map(|t| t.token_type) {
            self.consume();
            TokenType::NotEqual
        } else {
            TokenType::Bang
        };
        self.add_token(t);
        self.consume();
    }

    fn consume_digits(&mut self, digits: &'a str) {
        self.consume();
        self.add_token(TokenType::Integer(digits.parse().unwrap()));
    }

    fn add_indentation_error(&mut self, token: &Token1) {
        self.errors.add(super::indentation_error(token.pos))
    }
}

#[cfg(test)]
mod tests {
    use super::{Indentation, Pos, ToBigInt, Token, TokenType, TokenVec};

    fn without_pos<'a>(input: &TokenVec<'a>) -> TokenVec<'a> {
        let mut output: TokenVec = Vec::new();
        for token in input {
            output.push(Box::new(Token {
                pos: Pos::new(),
                token_type: token.token_type.clone(),
            }));
        }

        output
    }

    struct TestCase<'a> {
        pos: Pos,
        tokens: TokenVec<'a>,
    }

    impl<'a> TestCase<'a> {
        fn new() -> TestCase<'a> {
            TestCase {
                pos: Pos::new(),
                tokens: Vec::new(),
            }
        }

        fn token(mut self, token_type: TokenType<'a>) -> Self {
            self.tokens.push(Box::new(Token {
                pos: self.pos,
                token_type,
            }));
            self
        }

        fn indentation(mut self, tabs: usize, spaces: usize) -> Self {
            self.token(TokenType::Indent(Indentation::new(tabs, spaces)))
        }

        fn integer(mut self, n: usize) -> Self {
            self.token(TokenType::Integer(n.to_bigint().unwrap()))
        }

        fn ok_without_pos(&self, input: &str) {
            match super::tokenize(input) {
                Ok(tokens) => assert_eq!(without_pos(&tokens.tokens), self.tokens),
                Err(_) => assert!(false),
            }
        }
    }

    #[test]
    fn empty() {
        TestCase::new().ok_without_pos("");
    }

    #[test]
    fn digits() {
        TestCase::new()
            .indentation(0, 0)
            .integer(1234)
            .ok_without_pos("1234");
    }
}
