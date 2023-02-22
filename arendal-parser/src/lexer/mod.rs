use std::fmt;
use std::rc::Rc;

use super::{tokenizer, Enclosure, Errors, Integer, Pos, Result, Substr, Token, TokenKind, Tokens};

pub(crate) fn lex(input: &str) -> Result<Lexemes> {
    let tokens = tokenizer::tokenize(input)?;
    Lexer::new(tokens).lex()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct LexemeRef {
    lex_ref: Rc<Lexeme>,
}

impl LexemeRef {
    fn new(lexeme: Lexeme) -> Self {
        LexemeRef {
            lex_ref: Rc::new(lexeme),
        }
    }

    pub fn kind(&self) -> &LexemeKind {
        &self.lex_ref.kind
    }

    // Returns a clone of the position
    pub fn pos(&self) -> Pos {
        self.lex_ref.token.pos.clone()
    }
}

#[derive(Default, Clone)]
pub(crate) struct Lexemes {
    lexemes: Rc<Vec<LexemeRef>>,
}

impl Lexemes {
    fn new(lexref: &mut Vec<LexemeRef>) -> Self {
        let mut lexemes: Vec<LexemeRef> = Default::default();
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
    pub(crate) fn get(&self, index: usize) -> Option<LexemeRef> {
        self.lexemes.get(index).cloned()
    }

    pub(crate) fn merge<'a, I: IntoIterator<Item = Lexemes>>(values: I) -> Self {
        let mut lexemes: Vec<LexemeRef> = Default::default();
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
pub(crate) struct Lexeme {
    pub token: Token, // Starting token of the lexeme
    pub kind: LexemeKind,
}

impl Lexeme {
    fn new(token: &Token, kind: LexemeKind) -> Self {
        Lexeme {
            token: token.clone(),
            kind,
        }
    }
}

impl fmt::Debug for Lexeme {
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
    Equal,
    NotEquals,
    Integer(Integer),
    Open(Enclosure),
    Close(Enclosure),
    Underscore,
    Word(Substr),
}

struct Lexer {
    input: Tokens,
    lexemes: Vec<LexemeRef>,
    errors: Errors,
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
                TokenKind::Equal => self.add_lexeme(LexemeKind::Equal, 1),
                TokenKind::NotEquals => self.add_lexeme(LexemeKind::NotEquals, 1),
                TokenKind::Underscore => self.add_lexeme(LexemeKind::Underscore, 1),
                TokenKind::Open(e) => {
                    self.enclosures.push(e);
                    self.add_lexeme(LexemeKind::Open(e), 1)
                }
                TokenKind::Close(e) => self.add_close(&t, e),
                TokenKind::Digits(s) => self.add_digits(&s),
                _ => self.add_error(&t, Error::UnexpectedToken, 1),
            }
        }
        self.errors.to_result(Lexemes::new(&mut self.lexemes))
    }

    fn add_lexeme(&mut self, kind: LexemeKind, tokens: usize) {
        self.lexemes.push(LexemeRef::new(Lexeme::new(
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

    fn add_close(&mut self, token: &Token, e: Enclosure) {
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
                self.add_error(token, Error::InvalidClose(e), n);
            }
            None => {
                self.add_error(token, Error::InvalidClose(e), 1);
            }
        }
    }

    fn add_digits(&mut self, digits: &Substr) {
        self.add_lexeme(LexemeKind::Integer(digits.parse().unwrap()), 1);
    }

    fn add_error(&mut self, token: &Token, error: Error, tokens: usize) {
        self.errors.add(token.pos.clone().into(), error);
        self.advance(tokens)
    }
}

#[derive(Debug)]
enum Error {
    IndentationError,
    InvalidClose(Enclosure),
    UnexpectedToken,
}

impl super::Error for Error {}

#[cfg(test)]
mod tests;
