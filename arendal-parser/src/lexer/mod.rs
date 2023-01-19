use std::fmt;

use super::{
    error, tokenizer, BigInt, Errors, Indentation, NewLine, Pos, Result, Token, TokenKind, Tokens,
};

pub fn lex(input: &str) -> Result<Lexemes> {
    let pass1 = tokenizer::tokenize(input)?;
    lex2(pass1)
}

fn lex2(input: Tokens) -> Result<Lexemes> {
    Lexer::new(input).lex()
}

pub type Lexemes<'a> = Vec<Box<Lexeme<'a>>>;

#[derive(Clone, PartialEq, Eq)]
pub struct Lexeme<'a> {
    pub pos: Pos<'a>, // Starting position of the lexeme
    pub kind: LexemeKind<'a>,
}

impl<'a> fmt::Debug for Lexeme<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.kind, self.pos)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexemeKind<'a> {
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

impl<'a> LexemeKind<'a> {
    fn single(t: &TokenKind) -> Option<LexemeKind<'a>> {
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

struct Lexer<'a> {
    input: Tokens<'a>,
    lexemes: Lexemes<'a>,
    errors: Errors<'a>,
    index: usize,          // Index of the current input lexer
    lexeme_start: Pos<'a>, // Start of the current output lexer
}

impl<'a> Lexer<'a> {
    fn new(input: Tokens<'a>) -> Lexer<'a> {
        Lexer {
            input,
            lexemes: Vec::new(),
            errors: Default::default(),
            index: 0,
            lexeme_start: Pos::new(""),
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
    fn peek(&self) -> Option<Token<'a>> {
        self.input.get(self.index)
    }

    // Returns a clone of the lexer the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<Token<'a>> {
        self.input.get(self.index + n)
    }

    fn lex(mut self) -> Result<'a, Lexemes<'a>> {
        self.consume_indentation();
        while let Some(t) = self.peek() {
            self.lexeme_start = t.pos;
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
                TokenKind::Digits(s) => self.consume_digits(s),
                _ => self.errors.add(crate::parsing_error(t.pos)),
            }
        }
        self.errors.to_result(self.lexemes)
    }

    fn add_token(&mut self, token_type: LexemeKind<'a>) {
        self.lexemes.push(Box::new(Lexeme {
            pos: self.lexeme_start,
            kind: token_type,
        }));
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
        if let Some(token) = self.peek() {
            self.lexeme_start = token.pos;
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

    fn consume_digits(&mut self, digits: &'a str) {
        self.consume();
        self.add_token(LexemeKind::Integer(digits.parse().unwrap()));
    }

    fn add_error(&mut self, token: &Token<'a>, kind: ErrorKind) {
        self.errors.add(Error::new(token.clone(), kind))
    }
}

#[derive(Debug)]
struct Error<'a> {
    token: Token<'a>,
    kind: ErrorKind,
}

impl<'a> Error<'a> {
    fn new(token: Token<'a>, error_type: ErrorKind) -> Self {
        Error { token, kind: error_type }
    }
}

#[derive(Debug)]
enum ErrorKind {
    IndentationError,
}

impl<'a> error::Error<'a> for Error<'a> {}

#[cfg(test)]
mod tests;
