use std::fmt;
use std::rc::Rc;

use super::Enclosure;
use crate::tokenizer::{tokenize, Token, TokenKind, Tokens};
use core::error::{ErrorAcc, Loc, Result};
use core::identifier::{Id, TypeId};
use core::keyword::Keyword;
use core::{Integer, Substr};

pub(crate) fn lex(input: &str) -> Result<Lexemes> {
    let tokens = tokenize(input)?;
    Lexer::new(tokens).lex()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Lexeme {
    inner: Rc<Inner>,
}

impl Lexeme {
    fn new(lexeme: Inner) -> Self {
        Lexeme {
            inner: Rc::new(lexeme),
        }
    }

    pub fn kind(&self) -> &LexemeKind {
        &self.inner.kind
    }

    pub fn loc(&self) -> Loc {
        self.inner.token.loc()
    }
}

#[derive(Default, Clone)]
pub(crate) struct Lexemes {
    lexemes: Rc<Vec<Lexeme>>,
}

impl Lexemes {
    fn new(lexref: &mut Vec<Lexeme>) -> Self {
        let mut lexemes: Vec<Lexeme> = Default::default();
        lexemes.append(lexref);
        Lexemes {
            lexemes: Rc::new(lexemes),
        }
    }

    #[inline]
    pub(crate) fn contains(&self, index: usize) -> bool {
        index < self.lexemes.len()
    }

    #[inline]
    pub(crate) fn get(&self, index: usize) -> Option<Lexeme> {
        self.lexemes.get(index).cloned()
    }

    pub(crate) fn merge<'a, I: IntoIterator<Item = Lexemes>>(values: I) -> Self {
        let mut lexemes: Vec<Lexeme> = Default::default();
        values
            .into_iter()
            .for_each(|v| v.lexemes.iter().for_each(|l| lexemes.push(l.clone())));
        Self::new(&mut lexemes)
    }
}

impl fmt::Debug for Lexemes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.lexemes.as_ref())
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Inner {
    token: Token, // Starting token of the lexeme
    kind: LexemeKind,
}

impl Inner {
    fn new(token: &Token, kind: LexemeKind) -> Self {
        Inner {
            token: token.clone(),
            kind,
        }
    }
}

impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}[{:?}]", self.kind, self.token)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LexemeKind {
    Plus,
    Minus,
    Star,
    Slash,
    Dot,
    Greater,
    Less,
    Bang,
    Assignment,
    Equals,
    NotEquals,
    Integer(Integer),
    Open(Enclosure),
    Close(Enclosure),
    Underscore,
    Id(Id),
    TypeId(TypeId),
    Keyword(Keyword),
}

struct Lexer {
    input: Tokens,
    lexemes: Vec<Lexeme>,
    errors: ErrorAcc,
    index: usize,        // Index of the current input token
    lexeme_start: usize, // Index of the start token of the current lexeme
    enclosures: Vec<Enclosure>,
}

impl Lexer {
    fn new(input: Tokens) -> Lexer {
        Lexer {
            input,
            lexemes: Default::default(),
            errors: Default::default(),
            index: 0,
            lexeme_start: 0,
            enclosures: Default::default(),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        !self.input.contains(self.index)
    }

    // Advances the index the provided number of tokens.
    fn advance(&mut self, n: usize) {
        self.index += n;
    }

    // Returns a clone of the lexer at the current index, if any
    fn peek(&self) -> Option<Token> {
        self.input.get(self.index)
    }

    // Returns a clone of the lexer the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<Token> {
        self.input.get(self.index + n)
    }

    fn lex(mut self) -> Result<Lexemes> {
        while let Some(t) = self.peek() {
            self.lexeme_start = self.index;
            let loc = t.loc();
            match t.kind {
                TokenKind::Tabs(_) | TokenKind::Spaces(_) | TokenKind::EndOfLine(_) => {
                    self.advance_whitespace()
                }
                TokenKind::Plus => self.add_lexeme(LexemeKind::Plus, 1),
                TokenKind::Minus => self.add_lexeme(LexemeKind::Minus, 1),
                TokenKind::Star => self.add_lexeme(LexemeKind::Star, 1),
                TokenKind::Slash => self.add_lexeme(LexemeKind::Slash, 1),
                TokenKind::Dot => self.add_lexeme(LexemeKind::Dot, 1),
                TokenKind::Greater => self.add_lexeme(LexemeKind::Greater, 1),
                TokenKind::Less => self.add_lexeme(LexemeKind::Less, 1),
                TokenKind::Bang => self.add_lexeme(LexemeKind::Bang, 1),
                TokenKind::Assignment => self.add_lexeme(LexemeKind::Assignment, 1),
                TokenKind::Equals => self.add_lexeme(LexemeKind::Equals, 1),
                TokenKind::NotEquals => self.add_lexeme(LexemeKind::NotEquals, 1),
                TokenKind::Underscore => self.add_lexeme(LexemeKind::Underscore, 1),
                TokenKind::Open(e) => {
                    self.enclosures.push(e);
                    self.add_lexeme(LexemeKind::Open(e), 1)
                }
                TokenKind::Close(e) => self.add_close(loc, e),
                TokenKind::Digits(s) => self.add_digits(&s),
                TokenKind::Word(s) => self.add_word(loc, &s),
                _ => self.add_error(loc, Error::UnexpectedToken, 1),
            }
        }
        self.errors.to_result(Lexemes::new(&mut self.lexemes))
    }

    fn add_lexeme(&mut self, kind: LexemeKind, tokens: usize) {
        self.lexemes.push(Lexeme::new(Inner::new(
            &self.input.get(self.lexeme_start).unwrap(),
            kind,
        )));
        self.advance(tokens);
    }

    fn advance_whitespace(&mut self) {
        let mut n = 0;
        while let Some(t) = self.peek_ahead(n) {
            if t.is_whitespace() {
                n += 1;
            } else {
                break;
            }
        }
        if n > 0 {
            self.advance(n);
        }
    }

    fn add_close(&mut self, loc: Loc, e: Enclosure) {
        match self.enclosures.pop() {
            Some(last) => {
                if e == last {
                    self.add_lexeme(LexemeKind::Close(e), 1);
                    return;
                }
                // Advance until the right close token
                let mut n = 1;
                while let Some(t) = self.peek_ahead(n) {
                    if TokenKind::Close(e) == t.kind {
                        self.enclosures.push(e); // we add it again so that it is closed in the next cycle
                        break;
                    } else {
                        n += 1;
                    }
                }
                self.add_error(loc, Error::InvalidClose(e), n);
            }
            None => {
                self.add_error(loc, Error::InvalidClose(e), 1);
            }
        }
    }

    fn add_digits(&mut self, digits: &Substr) {
        self.add_lexeme(LexemeKind::Integer(digits.parse().unwrap()), 1);
    }

    fn add_word(&mut self, loc: Loc, word: &Substr) {
        if let Some(k) = Keyword::parse(word) {
            self.add_lexeme(LexemeKind::Keyword(k), 1);
        } else if let Ok(name) = TypeId::new(word.as_str().into()) {
            self.add_lexeme(LexemeKind::TypeId(name), 1);
        } else {
            match Id::new(word.as_str().into()) {
                Ok(name) => self.add_lexeme(LexemeKind::Id(name.into()), 1),
                Err(error) => self.errors.add(loc, error),
            }
        }
    }

    fn add_error(&mut self, loc: Loc, error: Error, tokens: usize) {
        self.errors.add(loc, error);
        self.advance(tokens)
    }
}

#[derive(Debug)]
enum Error {
    IndentationError,
    InvalidClose(Enclosure),
    UnexpectedToken,
}

impl core::error::Error for Error {}

#[cfg(test)]
mod tests;
