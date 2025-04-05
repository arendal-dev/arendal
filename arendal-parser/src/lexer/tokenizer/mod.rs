use std::fmt;
use std::rc::Rc;

use super::Enclosure;
use ast::input::{StrRange, StringInput};

pub(super) fn tokenize(input: StringInput) -> Tokens {
    Tokenizer::new(input).tokenize()
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Token {
    pub(super) range: StrRange,
    pub(super) kind: TokenKind,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)?;
        write!(
            f,
            "@{}-{}",
            self.range.from().bytes(),
            self.range.to().bytes()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Tokens {
    tokens: Rc<Vec<Token>>,
}

impl Tokens {
    pub(super) fn get(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenKind {
    Spaces,
    Tabs,
    NewLine,
    Plus,
    Minus,
    Star,
    Slash,
    Dot,
    Greater,
    GreaterOrEq,
    Less,
    LessOrEq,
    Bang,
    Assignment,
    Equals,
    NotEquals,
    Ampersand,
    LogicalAnd,
    Pipe,
    LogicalOr,
    Colon,
    DoubleColon,
    Open(Enclosure),
    Close(Enclosure),
    Underscore,
    Digits,
    Word,
    Other,
}

impl Token {
    pub fn is_whitespace(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Spaces | TokenKind::Tabs | TokenKind::NewLine
        )
    }

    fn chars(&self) -> usize {
        self.range.substr().chars().count()
    }

    fn bytes(&self) -> usize {
        self.range.substr().len()
    }
}

struct Tokenizer {
    chars: Vec<char>,
    index: usize,
    range: StrRange,
    tokens: Vec<Token>,
}

impl Tokenizer {
    fn new(input: StringInput) -> Tokenizer {
        let chars = input.as_char_vec();
        Tokenizer {
            chars,
            index: 0,
            range: StrRange::new(input),
            tokens: Vec::default(),
        }
    }

    fn advance(&mut self, c: char) {
        self.range.advance(c);
        self.index += 1;
    }

    fn advance_while<P>(&mut self, predicate: P)
    where
        P: Fn(char) -> bool,
    {
        while let Some(c) = self.peek() {
            if predicate(c) {
                self.advance(c);
            } else {
                break;
            }
        }
    }

    fn advance_while_char(&mut self, c: char) {
        self.advance_while(|n| n == c)
    }

    // Returns the char at the current index if any
    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn tokenize(mut self) -> Tokens {
        while let Some(c) = self.peek() {
            self.advance(c);
            if !self.add_known_first_char(c) && !self.add_digits(c) && !self.add_word(c) {
                self.add_token(TokenKind::Other);
            }
        }
        Tokens {
            tokens: Rc::new(self.tokens),
        }
    }

    fn add_known_first_char(&mut self, c: char) -> bool {
        match c {
            '\n' => self.add_token(TokenKind::NewLine),
            '\r' => self.add_token_if_next('\n', TokenKind::NewLine),
            ' ' => {
                self.advance_while_char(' ');
                self.add_token(TokenKind::Spaces)
            }
            '\t' => {
                self.advance_while_char('\t');
                self.add_token(TokenKind::Tabs)
            }
            '+' => self.add_token(TokenKind::Plus),
            '-' => self.add_token(TokenKind::Minus),
            '*' => self.add_token(TokenKind::Star),
            '/' => self.add_token(TokenKind::Slash),
            '.' => self.add_token(TokenKind::Dot),
            '>' => self.add_token_if_next_or_else('=', TokenKind::GreaterOrEq, TokenKind::Greater),
            '<' => self.add_token_if_next_or_else('=', TokenKind::LessOrEq, TokenKind::Less),
            '!' => self.add_token_if_next_or_else('=', TokenKind::NotEquals, TokenKind::Bang),
            '=' => self.add_token_if_next_or_else('=', TokenKind::Equals, TokenKind::Assignment),
            '&' => self.add_token_if_next_or_else('&', TokenKind::LogicalAnd, TokenKind::Ampersand),
            '|' => self.add_token_if_next_or_else('|', TokenKind::LogicalOr, TokenKind::Pipe),
            '(' => self.add_token(TokenKind::Open(Enclosure::Parens)),
            ')' => self.add_token(TokenKind::Close(Enclosure::Parens)),
            '{' => self.add_token(TokenKind::Open(Enclosure::Curly)),
            '}' => self.add_token(TokenKind::Close(Enclosure::Curly)),
            '[' => self.add_token(TokenKind::Open(Enclosure::Square)),
            ']' => self.add_token(TokenKind::Close(Enclosure::Square)),
            '_' => self.add_token(TokenKind::Underscore),
            ':' => self.add_token_if_next_or_else(':', TokenKind::DoubleColon, TokenKind::Colon),
            _ => false,
        }
    }

    fn add_digits(&mut self, c: char) -> bool {
        if c.is_ascii_digit() {
            self.advance_while(|n| n.is_ascii_digit());
            self.add_token(TokenKind::Digits)
        } else {
            false
        }
    }

    fn add_word(&mut self, c: char) -> bool {
        if c.is_ascii_alphabetic() {
            self.advance_while(|n| n.is_ascii_alphanumeric());
            self.add_token(TokenKind::Word)
        } else {
            false
        }
    }

    // Creates a token of the provided type consuming the needed chars.
    // Returns true to allow being the tail call of other add_ methods.
    fn add_token(&mut self, kind: TokenKind) -> bool {
        self.tokens.push(Token {
            range: self.range.clone(),
            kind,
        });
        self.range.catch_up();
        true
    }

    // Adds a token spanning two chars if the next one is the provided one
    fn add_token_if_next(&mut self, c: char, token: TokenKind) -> bool {
        if let Some(next) = self.peek() {
            if next == c {
                self.advance(c);
                return self.add_token(token);
            }
        }
        false
    }

    fn add_token_if_next_or_else(&mut self, c: char, token2: TokenKind, token1: TokenKind) -> bool {
        self.add_token_if_next(c, token2) || self.add_token(token1)
    }
}

#[cfg(test)]
mod tests;
