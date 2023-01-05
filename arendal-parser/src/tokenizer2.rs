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

pub type Tokens<'a> = Vec<Box<Token<'a>>>;

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pos: Pos<'a>, // Starting position of the token
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
    tokens: Tokens<'a>,
    errors: Errors<'a>,
    input_index: usize, // Index of the current input token
    token_start: Pos<'a>,   // Start of the current output token
}

impl<'a> Tokenizer<'a> {
    fn new(input: Tokens1<'a>) -> Tokenizer<'a> {
        Tokenizer {
            input,
            tokens: Vec::new(),
            errors: Errors::new(),
            input_index: 0,
            token_start: Pos::new(""),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.input_index >= self.input.len()
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
            Some(self.input[self.input_index].clone())
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
        if i >= self.input.len() {
            None
        } else {
            Some(self.input[i].clone())
        }
    }

    fn tokenize(mut self) -> Result<'a, Tokens<'a>> {
        self.consume_indentation();
        while let Some(t) = self.peek() {
            self.token_start = t.pos;
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
                _ => self.errors.add(super::unexpected_token(t.clone())),
            }
        }
        self.errors.to_result(self.tokens)
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

    fn consume_indentation(&mut self) {
        if let Some(token) = self.peek() {
            self.token_start = token.pos;
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
    }

    fn consume_eol(&mut self, token: Token1<'a>) {
        self.consume();
        self.consume_indentation();
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

    fn add_indentation_error(&mut self, token: &Token1<'a>) {
        self.errors.add(super::indentation_error(token.pos))
    }
}

#[cfg(test)]
mod tests {
    use super::{Indentation, Pos, ToBigInt, Token, TokenType, Tokens};

    fn eq_types(left: &Tokens, right: &Tokens) -> bool {
        let n = left.len();
        if n == right.len() {
            for (leftToken, rightToken) in left.iter().zip(right.iter()) {
                match &leftToken.token_type {
                    other => if *other != rightToken.token_type {
                        return false;
                    },
                }
            }
            true
        } else {
            false

        }
    }

    struct TestCase<'a> {
        input: &'a str,
        tokens: Tokens<'a>,
    }

    impl<'a> TestCase<'a> {
        fn new(input: &'a str) -> TestCase<'a> {
            TestCase {
                input,
                tokens: Vec::new(),
            }
        }

        fn token(mut self, token_type: TokenType<'a>) -> Self {
            self.tokens.push(Box::new(Token {
                pos: Pos::new(self.input),
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

        fn ok_without_pos(&self) {
            match super::tokenize(self.input) {
                Ok(tokens) => assert!(eq_types(&tokens, &self.tokens)),
                Err(_) => assert!(false),
            }
        }
    }

    #[test]
    fn empty() {
        TestCase::new("").ok_without_pos();
    }

    #[test]
    fn digits() {
        TestCase::new("1234")
            .indentation(0, 0)
            .integer(1234)
            .ok_without_pos();
        TestCase::new("\t1234")
            .indentation(1, 0)
            .integer(1234)
            .ok_without_pos();
    }
}
