use std::fmt;
use std::rc::Rc;

use super::{
    tokenizer, ArcStr, BigInt, Errors, Indentation, Loc, NewLine, Pos, Result, Substr, Token,
    TokenKind, Tokens,
};

pub fn lex(input: &str) -> Result<Lexemes> {
    let pass1 = tokenizer::tokenize(input)?;
    lex2(pass1)
}

fn lex2(input: Tokens) -> Result<Lexemes> {
    Lexer::new(input).lex()
}

#[derive(Debug, Clone)]
pub struct LexemeRef {
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
}

impl Loc for LexemeRef {}

#[derive(Default)]
pub struct Lexemes {
    lexemes: Vec<LexemeRef>,
}

impl Lexemes {
    #[inline]
    pub fn contains(&self, index: usize) -> bool {
        index < self.lexemes.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<LexemeRef> {
        self.lexemes.get(index).map(|l| l.clone())
    }
}

impl fmt::Debug for Lexemes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.lexemes)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Lexeme {
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
pub enum LexemeKind {
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
    Word(Substr),
}

impl LexemeKind {
    fn single(t: &TokenKind) -> Option<LexemeKind> {
        match t {
            TokenKind::Plus => Some(LexemeKind::Plus),
            TokenKind::Minus => Some(LexemeKind::Minus),
            TokenKind::Star => Some(LexemeKind::Star),
            TokenKind::Slash => Some(LexemeKind::Slash),
            TokenKind::Dot => Some(LexemeKind::Dot),
            TokenKind::Greater => Some(LexemeKind::Greater),
            TokenKind::Less => Some(LexemeKind::Less),
            TokenKind::Equal => Some(LexemeKind::Equal),
            TokenKind::Underscore => Some(LexemeKind::Underscore),
            _ => None,
        }
    }
}

struct Lexer {
    input: Tokens,
    lexemes: Lexemes,
    errors: Errors,
    index: usize,        // Index of the current input token
    lexeme_start: usize, // Index of the start token of the current lexeme
}

impl Lexer {
    fn new(input: Tokens) -> Lexer {
        Lexer {
            input,
            lexemes: Default::default(),
            errors: Default::default(),
            index: 0,
            lexeme_start: 0,
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        !self.input.contains(self.index)
    }

    // Consumes one lexer, advancing the index accordingly.
    fn consume(&mut self) {
        self.index += 1;
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
        self.consume_indentation();
        while let Some(t) = self.peek() {
            self.lexeme_start = self.index;
            if self.consume_whitespace(true) {
                continue;
            }
            if let Some(tt) = LexemeKind::single(&t.kind) {
                self.consume();
                self.add_token(tt);
                continue;
            }
            match t.kind {
                TokenKind::EndOfLine(_) => {
                    self.consume();
                    self.consume_indentation();
                }
                TokenKind::Bang => self.consume_bang(),
                TokenKind::Digits(s) => self.consume_digits(&s),
                _ => self.add_error(&t, ErrorKind::UnexpectedToken),
            }
        }
        self.errors.to_result(self.lexemes)
    }

    fn add_token(&mut self, token_type: LexemeKind) {
        self.lexemes.lexemes.push(LexemeRef::new(Lexeme::new(
            &self.input.get(self.lexeme_start).unwrap(),
            token_type,
        )));
    }

    fn consume_tabs(&mut self) -> usize {
        let mut tabs = 0;
        while let Some(Token {
            kind: TokenKind::Tabs(n),
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
        while let Some(Token {
            kind: TokenKind::Spaces(n),
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
            self.add_token(LexemeKind::Whitespace);
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
            if let TokenKind::EndOfLine(_) = token.kind {
                look_ahead += 1;
                remove += line_index + 1; // the EOL itself
                line_index = 0;
                continue;
            }
            break; // any other lexer
        }
        self.index += remove;
    }

    fn consume_indentation(&mut self) {
        self.skip_empty_lines();
        if let Some(_) = self.peek() {
            self.lexeme_start = self.index;
            let tabs = self.consume_tabs();
            let spaces = self.consume_spaces();
            self.add_token(LexemeKind::Indent(Indentation::new(tabs, spaces)));
            if let Some(t) = self.peek() {
                if t.is_whitespace() {
                    self.add_error(&t, ErrorKind::IndentationError);
                    self.consume_whitespace(false);
                }
            }
        }
    }

    fn consume_bang(&mut self) {
        let t = if let Some(TokenKind::Equal) = self.peek_ahead(1).map(|t| t.kind) {
            self.consume();
            LexemeKind::NotEqual
        } else {
            LexemeKind::Bang
        };
        self.add_token(t);
        self.consume();
    }

    fn consume_digits(&mut self, digits: &Substr) {
        self.consume();
        self.add_token(LexemeKind::Integer(digits.parse().unwrap()));
    }

    fn add_error(&mut self, token: &Token, kind: ErrorKind) {
        self.errors.add(Error::new(token.clone(), kind))
    }
}

#[derive(Debug)]
struct Error {
    token: Token,
    kind: ErrorKind,
}

impl Error {
    fn new(token: Token, error_type: ErrorKind) -> Self {
        Error {
            token,
            kind: error_type,
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    IndentationError,
    UnexpectedToken,
}

impl super::Error for Error {}

#[cfg(test)]
mod tests;
