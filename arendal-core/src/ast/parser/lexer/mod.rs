mod tokenizer;

use std::fmt;

use super::Enclosure;
use crate::error::{Error, Errors, Loc, Result};
use crate::keyword::Keyword;
use crate::symbol::{Symbol, TSymbol};
use crate::{Integer, Substr};
use tokenizer::{tokenize, Token, TokenKind, Tokens};

pub(super) fn lex(input: &str) -> Result<Lexemes> {
    let tokens = tokenize(input)?;
    Lexer::new(tokens).lex()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Separator {
    Nothing,
    Whitespace,
    NewLine,
}

impl Separator {
    fn add(self, other: Separator) -> Self {
        match self {
            Self::Nothing => other,
            Self::Whitespace => {
                if other == Self::NewLine {
                    Self::NewLine
                } else {
                    self
                }
            }
            Self::NewLine => self,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct Lexeme {
    pub(super) separator: Separator,
    token: Token, // Starting token of the lexeme
    pub(super) kind: LexemeKind,
}

impl Lexeme {
    fn new(separator: Separator, token: Token, kind: LexemeKind) -> Self {
        Lexeme {
            separator,
            token: token,
            kind,
        }
    }

    pub fn loc(&self) -> Loc {
        self.token.loc.clone()
    }
}

impl fmt::Debug for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}]{:?}[{:?}]", self.separator, self.kind, self.token)
    }
}

pub(super) type Lexemes = Vec<Lexeme>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LexemeKind {
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
    LogicalAnd,
    LogicalOr,
    Integer(Integer),
    Open(Enclosure),
    Close(Enclosure),
    Underscore,
    Symbol(Symbol),
    TSymbol(TSymbol),
    Keyword(Keyword),
}

struct Lexer {
    separator: Separator,
    input: Tokens,
    lexemes: Lexemes,
    errors: Errors,
    index: usize,        // Index of the current input token
    lexeme_start: usize, // Index of the start token of the current lexeme
    enclosures: Vec<Enclosure>,
}

impl Lexer {
    fn new(input: Tokens) -> Lexer {
        Lexer {
            separator: Separator::NewLine,
            input,
            lexemes: Default::default(),
            errors: Default::default(),
            index: 0,
            lexeme_start: 0,
            enclosures: Default::default(),
        }
    }

    // Advances the index the provided number of tokens.
    fn advance(&mut self, n: usize) {
        self.index += n;
    }

    // Returns a clone of the token at the current index, if any
    fn peek(&self) -> Option<Token> {
        self.input.get(self.index).cloned()
    }

    // Returns a clone of the lexer the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<Token> {
        self.input.get(self.index + n).cloned()
    }

    fn lex(mut self) -> Result<Lexemes> {
        self.separator = Separator::NewLine;
        while let Some(t) = self.peek() {
            self.lexeme_start = self.index;
            let loc = &t.loc;
            match t.kind {
                TokenKind::Tabs(_) | TokenKind::Spaces(_) => {
                    self.advance_whitespace();
                    self.separator = self.separator.add(Separator::Whitespace)
                }
                TokenKind::EndOfLine(_) => {
                    self.advance_whitespace();
                    self.separator = self.separator.add(Separator::NewLine)
                }
                TokenKind::Plus => self.add_lexeme(LexemeKind::Plus, 1),
                TokenKind::Minus => self.add_lexeme(LexemeKind::Minus, 1),
                TokenKind::Star => self.add_lexeme(LexemeKind::Star, 1),
                TokenKind::Slash => self.add_lexeme(LexemeKind::Slash, 1),
                TokenKind::Dot => self.add_lexeme(LexemeKind::Dot, 1),
                TokenKind::Greater => self.add_lexeme(LexemeKind::Greater, 1),
                TokenKind::GreaterOrEq => self.add_lexeme(LexemeKind::GreaterOrEq, 1),
                TokenKind::Less => self.add_lexeme(LexemeKind::Less, 1),
                TokenKind::LessOrEq => self.add_lexeme(LexemeKind::LessOrEq, 1),
                TokenKind::Bang => self.add_lexeme(LexemeKind::Bang, 1),
                TokenKind::Assignment => self.add_lexeme(LexemeKind::Assignment, 1),
                TokenKind::Equals => self.add_lexeme(LexemeKind::Equals, 1),
                TokenKind::LogicalAnd => self.add_lexeme(LexemeKind::LogicalAnd, 1),
                TokenKind::LogicalOr => self.add_lexeme(LexemeKind::LogicalOr, 1),
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
        self.errors.to_result(self.lexemes)
    }

    fn add_lexeme(&mut self, kind: LexemeKind, tokens: usize) {
        self.lexemes.push(Lexeme::new(
            self.separator,
            self.input.get(self.lexeme_start).cloned().unwrap(),
            kind,
        ));
        self.advance(tokens);
        self.separator = Separator::Nothing;
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

    fn add_close(&mut self, loc: &Loc, e: Enclosure) {
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

    fn add_word(&mut self, loc: &Loc, word: &Substr) {
        if let Some(k) = Keyword::parse(word) {
            self.add_lexeme(LexemeKind::Keyword(k), 1);
        } else if let Ok(name) = TSymbol::new(loc, word.as_str().into()) {
            self.add_lexeme(LexemeKind::TSymbol(name), 1);
        } else {
            if let Some(symbol) = self
                .errors
                .add_result(Symbol::new(loc, word.as_str().into()))
            {
                self.add_lexeme(LexemeKind::Symbol(symbol), 1);
            }
        }
    }

    fn add_error(&mut self, loc: &Loc, error: Error, tokens: usize) {
        self.errors.add(loc.error(error));
        self.advance(tokens)
    }
}

#[cfg(test)]
mod tests;
