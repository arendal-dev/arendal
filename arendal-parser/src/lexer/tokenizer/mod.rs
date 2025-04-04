use std::fmt::{self, Write};
use std::rc::Rc;

use super::Enclosure;
use ast::input::{NewLine, StrLen, StrRange, StringInput};
use ast::position::Position;
use ast::problem::{Problems, Result};
use arcstr::Substr;

pub(super) fn tokenize(input: StringInput) -> Result<Tokens> {
    Tokenizer::new(input).tokenize()
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Token {
    pub(super) position: Position,
    pub(super) data: TokenData,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)?;
        if let Position::String(r) = &self.position {
            write!(f, "@{}", r.from_bytes())
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(super) struct Tokens {
    tokens: Rc<Vec<Token>>
}

impl Tokens {
    pub(super) fn get(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenData {
    Spaces(usize),
    Tabs(usize),
    EndOfLine(NewLine),
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
    Digits(Substr),
    Word(Substr),
}

impl TokenData {
    fn len(&self) -> StrLen {
        match &self {
            TokenData::Spaces(n) => StrLen::new(*n, *n),
            TokenData::Tabs(n) => StrLen::new(*n, *n),
            TokenData::EndOfLine(nl) => nl.len(),
            TokenData::Digits(s) => StrLen::of_str(s.as_str()),
            TokenData::Word(s) => StrLen::of_str(s.as_str()),
            TokenData::NotEquals | TokenData::LogicalAnd | TokenData::LogicalOr | TokenData::GreaterOrEq | TokenData::LessOrEq | TokenData::DoubleColon => StrLen::new(2, 2),
            _ => StrLen::new(2, 2),
        }
    }
}

impl Token {
    pub fn is_whitespace(&self) -> bool {
        matches!(
            self.data,
            TokenData::Spaces(_) | TokenData::Tabs(_) | TokenData::EndOfLine(_)
        )
    }

    fn chars(&self) -> usize {
        match &self.data {
            TokenData::EndOfLine(nl) => nl.len().chars(),
            TokenData::Digits(s) => s.chars().count(),
            TokenData::Word(s) => s.chars().count(),
            _ => self.bytes(),
        }
    }

    fn bytes(&self) -> usize {
        match &self.data {
            TokenData::Spaces(n) => *n,
            TokenData::Tabs(n) => *n,
            TokenData::EndOfLine(nl) => nl.len().bytes(),
            TokenData::Digits(s) => s.len(),
            TokenData::Word(s) => s.len(),
            TokenData::NotEquals => 2,
            TokenData::LogicalAnd => 2,
            TokenData::LogicalOr => 2,
            TokenData::GreaterOrEq => 2,
            TokenData::LessOrEq => 2,
            TokenData::DoubleColon => 2,
            _ => 1,
        }
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

    fn advance_current_line(&mut self, len: StrLen) {
        self.range.advance(len);
        self.index += len.chars();
    }

    fn advance_to_new_line(&mut self, new_line: NewLine) {
        self.range.new_line(new_line);
        self.index += new_line.len().chars();
    }

    fn position(&mut self) -> Position {
        let position = Position::String(self.range.clone());
        self.range.catch_up();
        position
    }

    // Returns the char at the current index plus a certain offset if any
    fn peek(&self, offset: usize) -> Option<char> {
        let index = self.index + offset;
        if index >= self.chars.len() {
            None
        } else {
            Some(self.chars[index])
        }
    }

    fn peek_while<P>(&self, first: char, predicate: P) -> StrLen
    where
        P: Fn(char) -> bool,
    {
        let mut len = StrLen::of_char(first);
        while let Some(c) = self.peek(len.chars()) {
            if predicate(c) {
                len.add_char(c);
            } else {
                break;
            }
        }
        len
    }

    fn count_while(&self, c: char) -> usize {
        self.peek_while(c, |n| n == c).bytes()
    }

    fn tokenize(mut self) -> Result<Tokens> {
        let mut problems = Problems::default();
        while let Some(c) = self.peek(0) {
            if !self.add_known_first_char(c) && !self.add_digits(c) && !self.add_word(c) {
                self.advance_current_line(StrLen::of_char(c));
                problems.add_error(self.position(), "E10101", format!("Unexpected char: {}", c));
            }
        }
        problems.to_lazy_result(|| Tokens { tokens: Rc::new(self.tokens) })
    }

    fn add_known_first_char(&mut self, c: char) -> bool {
        match c {
            '\n' => self.add_token(TokenData::EndOfLine(NewLine::LF)),
            '\r' => self.add_token_if_next('\n', TokenData::EndOfLine(NewLine::CRLF)),
            ' ' => self.add_token(TokenData::Spaces(self.count_while(' '))),
            '\t' => self.add_token(TokenData::Tabs(self.count_while('\t'))),
            '+' => self.add_token(TokenData::Plus),
            '-' => self.add_token(TokenData::Minus),
            '*' => self.add_token(TokenData::Star),
            '/' => self.add_token(TokenData::Slash),
            '.' => self.add_token(TokenData::Dot),
            '>' => self.add_token_if_next_or_else('=', TokenData::GreaterOrEq, TokenData::Greater),
            '<' => self.add_token_if_next_or_else('=', TokenData::LessOrEq, TokenData::Less),
            '!' => self.add_token_if_next_or_else('=', TokenData::NotEquals, TokenData::Bang),
            '=' => self.add_token_if_next_or_else('=', TokenData::Equals, TokenData::Assignment),
            '&' => self.add_token_if_next_or_else('&', TokenData::LogicalAnd, TokenData::Ampersand),
            '|' => self.add_token_if_next_or_else('|', TokenData::LogicalOr, TokenData::Pipe),
            '(' => self.add_token(TokenData::Open(Enclosure::Parens)),
            ')' => self.add_token(TokenData::Close(Enclosure::Parens)),
            '{' => self.add_token(TokenData::Open(Enclosure::Curly)),
            '}' => self.add_token(TokenData::Close(Enclosure::Curly)),
            '[' => self.add_token(TokenData::Open(Enclosure::Square)),
            ']' => self.add_token(TokenData::Close(Enclosure::Square)),
            '_' => self.add_token(TokenData::Underscore),
            ':' => self.add_token_if_next_or_else(':', TokenData::DoubleColon, TokenData::Colon),
            _ => false,
        }
    }

    fn add_digits(&mut self, c: char) -> bool {
        if c.is_ascii_digit() {
            let len = self.peek_while(c, |n| n.is_ascii_digit());
            self.add_token(TokenData::Digits(self.range.get_substr_len(len)))
        } else {
            false
        }
    }

    fn add_word(&mut self, c: char) -> bool {
        if c.is_ascii_alphabetic() {
            let len = self.peek_while(c, |n| n.is_ascii_alphanumeric());
            self.add_token(TokenData::Word(self.range.get_substr_len(len)))
        } else {
            false
        }
    }

    // Creates a token of the provided type consuming the needed chars.
    // Returns true to allow being the tail call of other add_ methods.
    fn add_token(&mut self, data: TokenData) -> bool {
        match &data {
            TokenData::EndOfLine(nl) => self.advance_to_new_line(*nl),
            d => self.advance_current_line(d.len()),
        }
        let position = self.position();
        self.tokens.push(Token { position, data });
        true
    }

    fn add_token_if_next(&mut self, c: char, token: TokenData) -> bool {
        if let Some(next) = self.peek(1) {
            if next == c {
                return self.add_token(token);
            }
        }
        false
    }

    fn add_token_if_next_or_else(&mut self, c: char, token2: TokenData, token1: TokenData) -> bool {
        self.add_token_if_next(c, token2) || self.add_token(token1)
    }
}

#[cfg(test)]
mod tests;
