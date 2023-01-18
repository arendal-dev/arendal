use super::{Enclosure, Errors, NewLine, Pos, Result};
use std::fmt;

pub fn tokenize(input: &str) -> Result<Tokens> {
    Tokenizer::new(input).tokenize()
}

#[derive(Default, PartialEq, Eq)]
pub struct Tokens<'a> {
    tokens: Vec<Token<'a>>,
}

impl<'a> Tokens<'a> {
    #[inline]
    pub fn contains(&self, index: usize) -> bool {
        index < self.tokens.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<Token<'a>> {
        self.tokens.get(index).map(|t| t.clone())
    }
}

impl<'a> fmt::Debug for Tokens<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.tokens)
    }
}


#[derive(Clone, PartialEq, Eq)]
pub struct Token<'a> {
    pub pos: Pos<'a>, // Starting position of the lexer
    pub kind: TokenKind<'a>,
}

impl<'a> Token<'a> {
    pub fn is_whitespace(&self) -> bool {
        self.kind.is_whitespace()
    }
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.kind, self.pos)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind<'a> {
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
    Open(Enclosure),
    Close(Enclosure),
    Underscore,
    Digits(&'a str),
    Word(&'a str),
}

impl<'a> TokenKind<'a> {
    fn is_whitespace(&self) -> bool {
        matches!(self, TokenKind::Spaces(_) | TokenKind::Tabs(_))
    }

    fn single(c: char) -> Option<TokenKind<'a>> {
        match c {
            '+' => Some(TokenKind::Plus),
            '-' => Some(TokenKind::Minus),
            '*' => Some(TokenKind::Star),
            '/' => Some(TokenKind::Slash),
            '.' => Some(TokenKind::Dot),
            '>' => Some(TokenKind::Greater),
            '<' => Some(TokenKind::Less),
            '!' => Some(TokenKind::Bang),
            '=' => Some(TokenKind::Equal),
            '(' => Some(TokenKind::Open(Enclosure::Parens)),
            ')' => Some(TokenKind::Close(Enclosure::Parens)),
            '{' => Some(TokenKind::Open(Enclosure::Curly)),
            '}' => Some(TokenKind::Close(Enclosure::Curly)),
            '[' => Some(TokenKind::Open(Enclosure::Square)),
            ']' => Some(TokenKind::Close(Enclosure::Square)),
            '_' => Some(TokenKind::Underscore),
            _ => None,
        }
    }
}

struct Tokenizer<'a> {
    chars: Vec<char>,
    tokens: Tokens<'a>,
    errors: Errors<'a>,
    // Positions
    pos: Pos<'a>,
    // Current char index from the beginning of the input
    char_index: usize,
    // Start of the current token
    token_start: Pos<'a>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &str) -> Tokenizer {
        Tokenizer {
            chars: input.chars().collect(),
            tokens: Default::default(),
            errors: Default::default(),
            pos: Pos::new(input),
            char_index: 0,
            token_start: Pos::new(input),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.char_index >= self.chars.len()
    }

    // Consumes one char, advancing the indices accordingly.
    fn consume(&mut self) {
        let bytes = self.chars[self.char_index].len_utf8();
        self.pos.advance(bytes);
        self.char_index += 1;
    }

    // Returns the char at the current index.
    // Panics if we have reached the end of the input
    fn peek(&self) -> char {
        self.chars[self.char_index]
    }

    // Returns true if there's a next char and it's equal to the provided one.
    fn next_matches(&self, c: char) -> bool {
        let i = self.char_index + 1;
        if i >= self.chars.len() {
            false
        } else {
            c == self.chars[i]
        }
    }

    fn tokenize(mut self) -> Result<'a, Tokens<'a>> {
        while !self.is_done() {
            self.token_start = self.pos;
            let c = self.peek();
            if let Some(t) = TokenKind::single(c) {
                self.consume_single_char_token(t);
            } else {
                match c {
                    ' ' => self.consume_spaces(),
                    '\t' => self.consume_tabs(),
                    _ => self.tokenize2(c),
                }
            }
        }
        self.errors.to_result(self.tokens)
    }

    fn tokenize2(&mut self, c: char) {
        if !self.consume_eol(c) && !self.consume_digits(c) && !self.consume_word(c) {
            self.add_unexpected_char(c)
        }
    }

    // Consumes a char, creating the provided lexer
    fn consume_single_char_token(&mut self, token_type: TokenKind<'a>) {
        self.consume();
        self.add_token(token_type);
    }

    // Returns whether the provided char is a newline, peeking the next if needed
    // Consumes a new line if found in the current position
    fn is_eol(&self, c: char) -> Option<NewLine> {
        if c == '\n' {
            Some(NewLine::LF)
        } else if c == '\r' && self.next_matches('\n') {
            Some(NewLine::CRLF)
        } else {
            None
        }
    }

    // Consumes a new line if found in the current position
    fn consume_eol(&mut self, c: char) -> bool {
        match self.is_eol(c) {
            Some(nl) => {
                self.pos.advance(nl.bytes());
                self.char_index += nl.chars();
                self.add_token(TokenKind::EndOfLine(nl));
                true
            }
            _ => false,
        }
    }

    // Starts a lexer a consumes chars while they are equal to the one provided.
    // Returns the number of chars consumed.
    fn consume_multiple(&mut self, c: char) -> usize {
        let mut count = 1;
        self.consume();
        while !self.is_done() && self.peek() == c {
            self.consume();
            count += 1
        }
        count
    }

    fn consume_spaces(&mut self) {
        let token = TokenKind::Spaces(self.consume_multiple(' '));
        self.add_token(token);
    }

    fn consume_tabs(&mut self) {
        let token = TokenKind::Tabs(self.consume_multiple('\t'));
        self.add_token(token);
    }

    fn consume_digits(&mut self, mut c: char) -> bool {
        let mut consumed = false;
        while c.is_ascii_digit() {
            self.consume();
            consumed = true;
            if self.is_done() {
                break;
            } else {
                c = self.peek();
            }
        }
        if consumed {
            self.add_token(TokenKind::Digits(self.get_token_str()));
        }
        consumed
    }

    fn consume_word(&mut self, mut c: char) -> bool {
        if !c.is_ascii_alphabetic() {
            return false;
        }
        let mut consumed = false;
        while c.is_ascii_alphanumeric() {
            self.consume();
            consumed = true;
            if self.is_done() {
                break;
            } else {
                c = self.peek();
            }
        }
        if consumed {
            self.add_token(TokenKind::Word(self.get_token_str()));
        }
        consumed
    }

    fn get_token_str(&self) -> &'a str {
        self.token_start.str_to(&self.pos)
    }

    fn add_token(&mut self, token_type: TokenKind<'a>) {
        self.tokens.tokens.push(Token {
            pos: self.token_start,
            kind: token_type,
        });
    }

    fn add_unexpected_char(&mut self, c: char) {
        self.errors.add(super::unexpected_char(self.pos, c))
    }
}

#[cfg(test)]
mod tests;
