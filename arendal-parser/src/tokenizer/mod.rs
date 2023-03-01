use super::{ArcStr, Enclosure, Errors, Loc, Pos, Result, Substr};
use std::fmt;

pub(crate) fn tokenize(input: &str) -> Result<Tokens> {
    Tokenizer::new(ArcStr::from(input)).tokenize()
}

#[derive(Default, PartialEq, Eq)]
pub(crate) struct Tokens {
    tokens: Vec<Token>,
}

impl Tokens {
    #[inline]
    pub fn contains(&self, index: usize) -> bool {
        index < self.tokens.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<Token> {
        self.tokens.get(index).cloned()
    }
}

impl fmt::Debug for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.tokens)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub(crate) pos: Pos, // Starting position of the token
    pub(crate) kind: TokenKind,
}

impl Token {
    pub fn is_whitespace(&self) -> bool {
        self.kind.is_whitespace()
    }

    pub fn loc(&self) -> Loc {
        self.pos.clone().into()
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.kind, self.pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NewLine {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
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
    NotEquals,
    Open(Enclosure),
    Close(Enclosure),
    Underscore,
    Digits(Substr),
    Word(Substr),
}

impl TokenKind {
    fn is_whitespace(&self) -> bool {
        matches!(
            self,
            TokenKind::Spaces(_) | TokenKind::Tabs(_) | TokenKind::EndOfLine(_)
        )
    }

    fn chars(&self) -> usize {
        match self {
            TokenKind::Spaces(n) => *n,
            TokenKind::Tabs(n) => *n,
            TokenKind::EndOfLine(nl) => nl.chars(),
            TokenKind::Digits(s) => s.chars().count(),
            TokenKind::Word(s) => s.chars().count(),
            TokenKind::NotEquals => 2,
            _ => 1,
        }
    }

    fn bytes(&self) -> usize {
        match self {
            TokenKind::Spaces(n) => *n,
            TokenKind::Tabs(n) => *n,
            TokenKind::EndOfLine(nl) => nl.bytes(),
            TokenKind::Digits(s) => s.len(),
            TokenKind::Word(s) => s.len(),
            TokenKind::NotEquals => 2,
            _ => 1,
        }
    }
}

struct Tokenizer {
    chars: Vec<char>,
    tokens: Tokens,
    errors: Errors,
    pos: Pos,          // Current position
    char_index: usize, // Current char index from the beginning of the input
}

impl Tokenizer {
    fn new(input: ArcStr) -> Tokenizer {
        let pos = Pos::new(input.clone());
        Tokenizer {
            chars: input.chars().collect(),
            tokens: Default::default(),
            errors: Default::default(),
            pos,
            char_index: 0,
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.char_index >= self.chars.len()
    }

    // Consumes one char, advancing the indices accordingly.
    fn consume(&mut self) {
        self.pos.advance_char(self.chars[self.char_index]);
        self.char_index += 1;
    }

    // Consumes multiple chars, advancing the indices accordingly.
    fn consume_chars(&mut self, n: usize) {
        for _ in 0..n {
            self.consume();
        }
    }

    // Returns the char at the current index if any
    fn peek(&self) -> Option<char> {
        if self.is_done() {
            None
        } else {
            Some(self.chars[self.char_index])
        }
    }

    // Returns the char at the current index if any
    fn peek_ahead(&self, n: usize) -> Option<char> {
        let index = self.char_index + n;
        if index >= self.chars.len() {
            None
        } else {
            Some(self.chars[index])
        }
    }

    fn count_while(&self, c: char) -> usize {
        let mut n = 1;
        while let Some(ahead) = self.peek_ahead(n) {
            if c == ahead {
                n += 1;
            } else {
                break;
            }
        }
        n
    }

    // Creates a substring of the input that starts at the current position, includes the initial
    // char and any subsequent char until the predicate is false.
    fn substr_while<P>(&self, initial: char, predicate: P) -> Substr
    where
        P: Fn(char) -> bool,
    {
        let mut n = 1;
        let mut pos = self.pos.clone();
        pos.advance_char(initial);
        while let Some(c) = self.peek_ahead(n) {
            if predicate(c) {
                n += 1;
                pos.advance_char(c);
            } else {
                break;
            }
        }
        self.pos.str_to(&pos)
    }

    fn tokenize(mut self) -> Result<Tokens> {
        while let Some(c) = self.peek() {
            if !self.add_known_first_char(c) && !self.add_digits(c) && !self.add_word(c) {
                self.add_error(Error::UnexpectedChar(c));
                self.consume();
            }
        }
        self.errors.to_result(self.tokens)
    }

    fn add_known_first_char(&mut self, c: char) -> bool {
        match c {
            '\n' => self.add_token(TokenKind::EndOfLine(NewLine::LF)),
            '\r' => self.add_token_if_next('\n', TokenKind::EndOfLine(NewLine::CRLF)),
            ' ' => self.add_token(TokenKind::Spaces(self.count_while(' '))),
            '\t' => self.add_token(TokenKind::Tabs(self.count_while('\t'))),
            '+' => self.add_token(TokenKind::Plus),
            '-' => self.add_token(TokenKind::Minus),
            '*' => self.add_token(TokenKind::Star),
            '/' => self.add_token(TokenKind::Slash),
            '.' => self.add_token(TokenKind::Dot),
            '>' => self.add_token(TokenKind::Greater),
            '<' => self.add_token(TokenKind::Less),
            '!' => self.add_token_if_next_or_else('=', TokenKind::NotEquals, TokenKind::Bang),
            '=' => self.add_token(TokenKind::Equal),
            '(' => self.add_token(TokenKind::Open(Enclosure::Parens)),
            ')' => self.add_token(TokenKind::Close(Enclosure::Parens)),
            '{' => self.add_token(TokenKind::Open(Enclosure::Curly)),
            '}' => self.add_token(TokenKind::Close(Enclosure::Curly)),
            '[' => self.add_token(TokenKind::Open(Enclosure::Square)),
            ']' => self.add_token(TokenKind::Close(Enclosure::Square)),
            '_' => self.add_token(TokenKind::Underscore),
            _ => false,
        }
    }

    fn add_digits(&mut self, c: char) -> bool {
        if c.is_ascii_digit() {
            self.add_token(TokenKind::Digits(
                self.substr_while(c, |n| n.is_ascii_digit()),
            ))
        } else {
            false
        }
    }

    fn add_word(&mut self, c: char) -> bool {
        if c.is_ascii_alphabetic() {
            self.add_token(TokenKind::Word(
                self.substr_while(c, |n| n.is_ascii_alphanumeric()),
            ))
        } else {
            false
        }
    }

    // Creates a token of the provided type consuming the needed chars.
    // Returns true to allow being the tail call of other add_ methods.
    fn add_token(&mut self, kind: TokenKind) -> bool {
        let chars = kind.chars();
        self.tokens.tokens.push(Token {
            pos: self.pos.clone(),
            kind,
        });
        self.consume_chars(chars);
        true
    }

    fn add_token_if_next(&mut self, c: char, kind: TokenKind) -> bool {
        if let Some(next) = self.peek_ahead(1) {
            if next == c {
                return self.add_token(kind);
            }
        }
        false
    }

    fn add_token_if_next_or_else(&mut self, c: char, kind2: TokenKind, kind1: TokenKind) -> bool {
        self.add_token_if_next(c, kind2) || self.add_token(kind1)
    }

    fn add_error(&mut self, error: Error) {
        self.errors.add(self.pos.clone().into(), error);
    }
}

#[derive(Debug)]
enum Error {
    UnexpectedChar(char),
}

impl super::Error for Error {}

#[cfg(test)]
mod tests;
