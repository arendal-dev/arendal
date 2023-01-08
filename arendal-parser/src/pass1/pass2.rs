mod parser;

use super::{CharToken, CharTokenType, CharTokens, Errors, NewLine, Pos, Result};
use crate::Indentation;
use num::bigint::{BigInt, ToBigInt};
use std::fmt;

fn tokenize(input: &str) -> Result<Tokens> {
    let pass1 = super::tokenize(input)?;
    tokenize2(pass1)
}

fn tokenize2(input: CharTokens) -> Result<Tokens> {
    Tokenizer::new(input).tokenize()
}

type Tokens<'a> = Vec<Box<Token<'a>>>;

#[derive(Clone, PartialEq, Eq)]
struct Token<'a> {
    pos: Pos<'a>, // Starting position of the token
    token_type: TokenType<'a>,
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.token_type, self.pos)
    }
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
    fn single(t: &CharTokenType) -> Option<TokenType<'a>> {
        match t {
            CharTokenType::Plus => Some(TokenType::Plus),
            CharTokenType::Minus => Some(TokenType::Minus),
            CharTokenType::Star => Some(TokenType::Star),
            CharTokenType::Slash => Some(TokenType::Slash),
            CharTokenType::Dot => Some(TokenType::Dot),
            CharTokenType::Greater => Some(TokenType::Greater),
            CharTokenType::Less => Some(TokenType::Less),
            CharTokenType::Equal => Some(TokenType::Equal),
            CharTokenType::Underscore => Some(TokenType::Underscore),
            _ => None,
        }
    }
}

struct Tokenizer<'a> {
    input: CharTokens<'a>,
    tokens: Tokens<'a>,
    errors: Errors<'a>,
    index: usize,         // Index of the current input token
    token_start: Pos<'a>, // Start of the current output token
}

impl<'a> Tokenizer<'a> {
    fn new(input: CharTokens<'a>) -> Tokenizer<'a> {
        Tokenizer {
            input,
            tokens: Vec::new(),
            errors: Errors::new(),
            index: 0,
            token_start: Pos::new(""),
        }
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
    fn peek(&self) -> Option<CharToken<'a>> {
        if self.is_done() {
            None
        } else {
            Some(self.input[self.index].clone())
        }
    }

    // Returns a clone of the token the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<CharToken<'a>> {
        let i = self.index + n;
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
                CharTokenType::EndOfLine(_) => {
                    self.consume();
                    self.consume_indentation();
                }
                CharTokenType::Bang => self.consume_bang(),
                CharTokenType::Digits(s) => self.consume_digits(s),
                _ => self.errors.add(crate::parsing_error(t.pos)),
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

    fn consume_tabs(&mut self) -> usize {
        let mut tabs = 0;
        while let Some(CharToken {
            token_type: CharTokenType::Tabs(n),
            ..
        }) = self.peek()
        {
            tabs += n;
            self.consume();
        }
        tabs
    }

    fn consume_spaces(&mut self) -> usize {
        let mut spaces = 0;
        while let Some(CharToken {
            token_type: CharTokenType::Spaces(n),
            ..
        }) = self.peek()
        {
            spaces += n;
            self.consume();
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

    fn skip_empty_lines(&mut self) {
        // Skip empty lines
        let mut look_ahead: usize = 0;
        let mut remove: usize = 0;
        let mut line_index: usize = 0;
        while let Some(token) = self.peek_ahead(look_ahead) {
            if token.is_whitespace() {
                look_ahead += 1;
                line_index += 1;
                continue;
            }
            if let CharTokenType::EndOfLine(_) = token.token_type {
                look_ahead += 1;
                remove += line_index + 1; // the EOL itself
                line_index = 0;
                continue;
            }
            break; // any other token
        }
        self.index += remove;
    }

    fn consume_indentation(&mut self) {
        self.skip_empty_lines();
        if let Some(token) = self.peek() {
            self.token_start = token.pos;
            let tabs = self.consume_tabs();
            let spaces = self.consume_spaces();
            self.add_token(TokenType::Indent(Indentation::new(tabs, spaces)));
            if let Some(t) = self.peek() {
                if t.is_whitespace() {
                    self.add_indentation_error(&t);
                    self.consume_whitespace(false);
                }
            }
        }
    }

    fn consume_bang(&mut self) {
        let t = if let Some(CharTokenType::Equal) = self.peek_ahead(1).map(|t| t.token_type) {
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

    fn add_indentation_error(&mut self, token: &CharToken<'a>) {
        self.errors.add(crate::indentation_error(token.pos))
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
                    other => {
                        if *other != rightToken.token_type {
                            return false;
                        }
                    }
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

        fn whitespace(mut self) -> Self {
            self.token(TokenType::Whitespace)
        }

        fn indentation(mut self, tabs: usize, spaces: usize) -> Self {
            self.token(TokenType::Indent(Indentation::new(tabs, spaces)))
        }

        fn integer(mut self, n: usize) -> Self {
            self.token(TokenType::Integer(n.to_bigint().unwrap()))
        }

        fn ok_without_pos(&self) {
            match super::tokenize(self.input) {
                Ok(tokens) => assert!(
                    eq_types(&tokens, &self.tokens),
                    "{:?}\n{:?}",
                    &tokens,
                    &self.tokens
                ),
                Err(_) => assert!(false),
            }
        }
    }

    #[test]
    fn empty() {
        TestCase::new("").ok_without_pos();
    }

    #[test]
    fn digits1() {
        TestCase::new("1234")
            .indentation(0, 0)
            .integer(1234)
            .ok_without_pos();
    }

    #[test]
    fn digits2() {
        TestCase::new("\t1234")
            .indentation(1, 0)
            .integer(1234)
            .ok_without_pos();
    }

    #[test]
    fn digits3() {
        TestCase::new("\t 1234")
            .indentation(1, 1)
            .integer(1234)
            .ok_without_pos();
    }

    #[test]
    fn sum1() {
        TestCase::new("1234+456")
            .indentation(0, 0)
            .integer(1234)
            .token(TokenType::Plus)
            .integer(456)
            .ok_without_pos();
    }

    #[test]
    fn sum2() {
        TestCase::new("  1234 +  456")
            .indentation(0, 2)
            .integer(1234)
            .whitespace()
            .token(TokenType::Plus)
            .whitespace()
            .integer(456)
            .ok_without_pos();
    }

    #[test]
    fn sum3() {
        TestCase::new("  1234 +\n\t456")
            .indentation(0, 2)
            .integer(1234)
            .whitespace()
            .token(TokenType::Plus)
            .indentation(1, 0)
            .integer(456)
            .ok_without_pos();
    }

    #[test]
    fn remove_empty_lines1() {
        TestCase::new("\n\n \n1234")
            .indentation(0, 0)
            .integer(1234)
            .ok_without_pos();
    }

    #[test]
    fn remove_empty_lines2() {
        TestCase::new("\n\n \n\t \n \t \n1234")
            .indentation(0, 0)
            .integer(1234)
            .ok_without_pos();
    }

    #[test]
    fn remove_empty_lines3() {
        TestCase::new("\n\n \n\t \n \t \n\t 1234")
            .indentation(1, 1)
            .integer(1234)
            .ok_without_pos();
    }

    #[test]
    fn remove_empty_lines4() {
        TestCase::new("\n\n \n\t \n \t \n\t 1234\n\n \n 567\n\n")
            .indentation(1, 1)
            .integer(1234)
            .indentation(0, 1)
            .integer(567)
            .ok_without_pos();
    }
}
