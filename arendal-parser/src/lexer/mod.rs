mod tokenizer;

use std::fmt;

use ast::input::StringInput;
use ast::keyword::Keyword;
use ast::position::{EqNoPosition, Position};
use ast::problem::{ErrorType, Output};
use ast::symbol::{Symbol, TSymbol};
use num::Integer;
use tokenizer::{Token, TokenKind, Tokens, tokenize};

pub(super) fn lex(input: StringInput) -> Output<Lexemes> {
    let tokens = tokenize(input);
    Lexer::new(&tokens).lex().0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Enclosure {
    Parens,
    Square,
    Curly,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Separator {
    Start,
    Nothing,
    Whitespace,
    NewLine,
}

impl std::ops::Add for Separator {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Self::Start | Self::Nothing => other,
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

#[derive(Eq, PartialEq)]
pub(super) struct Lexeme {
    pub(super) position: Position,
    pub(super) separator: Separator,
    pub(super) data: LexemeData,
}

impl fmt::Debug for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{:?}{}]", self.separator, self.data, self.position)
    }
}

impl EqNoPosition for Lexeme {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.separator == other.separator
            && match &self.data {
                LexemeData::Level(level1) => match &other.data {
                    LexemeData::Level(level2) => {
                        level1.enclosure == level2.enclosure
                            && level1.lexemes.lexemes.eq_nopos(&level2.lexemes.lexemes)
                    }
                    _ => false,
                },
                _ => self.data == other.data,
            }
    }
}

#[derive(Default, Eq, PartialEq)]
pub(super) struct Lexemes {
    lexemes: Vec<Lexeme>,
}

impl Lexemes {
    pub(super) fn get(&self, index: usize) -> Option<&Lexeme> {
        self.lexemes.get(index)
    }

    pub(super) fn len(&self) -> usize {
        self.lexemes.len()
    }
}

impl fmt::Debug for Lexemes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self.lexemes).fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Level {
    pub(super) enclosure: Enclosure,
    pub(super) lexemes: Lexemes,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum LexemeData {
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
    TypeAnnSeparator,
    PathSeparator,
    Integer(Integer),
    Level(Level),
    Underscore,
    Symbol(Symbol),
    TSymbol(TSymbol),
    Keyword(Keyword),
}

struct Lexer<'me> {
    separator: Separator,
    tokens: &'me Tokens,
    output: Output<Vec<Lexeme>>,
    index: usize,        // Index of the current input token
    lexeme_start: usize, // Index of the start token of the current lexeme
    enclosed_by: Option<Enclosure>,
}

#[derive(Debug)]
enum Error {
    InvalidWord,
    NoOpenEnclosure,
    InvalidOpenEnclosure,
}

impl ErrorType for Error {}

impl<'me> Lexer<'me> {
    fn new(tokens: &Tokens) -> Lexer {
        Lexer {
            separator: Separator::Start,
            tokens,
            output: Output::empty(),
            index: 0,
            lexeme_start: 0,
            enclosed_by: None,
        }
    }

    // Advances the index the provided number of tokens.
    fn advance(&mut self, n: usize) {
        self.index += n;
    }

    // Returns the token at the current index, if any
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    // Returns the token the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.index + n)
    }

    fn output(self) -> (Output<Lexemes>, usize) {
        (self.output.map(|v| Lexemes { lexemes: v }), self.index)
    }

    fn lex(mut self) -> (Output<Lexemes>, usize) {
        while let Some(t) = self.peek().cloned() {
            self.lexeme_start = self.index;
            match t.kind {
                TokenKind::Tabs | TokenKind::Spaces => {
                    self.advance_whitespace(Separator::Whitespace)
                }
                TokenKind::NewLine => self.advance_whitespace(Separator::NewLine),
                TokenKind::Plus => self.add_lexeme(LexemeData::Plus, 1),
                TokenKind::Minus => self.add_lexeme(LexemeData::Minus, 1),
                TokenKind::Star => self.add_lexeme(LexemeData::Star, 1),
                TokenKind::Slash => self.add_lexeme(LexemeData::Slash, 1),
                TokenKind::Dot => self.add_lexeme(LexemeData::Dot, 1),
                TokenKind::Greater => self.add_lexeme(LexemeData::Greater, 1),
                TokenKind::GreaterOrEq => self.add_lexeme(LexemeData::GreaterOrEq, 1),
                TokenKind::Less => self.add_lexeme(LexemeData::Less, 1),
                TokenKind::LessOrEq => self.add_lexeme(LexemeData::LessOrEq, 1),
                TokenKind::Bang => self.add_lexeme(LexemeData::Bang, 1),
                TokenKind::Assignment => self.add_lexeme(LexemeData::Assignment, 1),
                TokenKind::Equals => self.add_lexeme(LexemeData::Equals, 1),
                TokenKind::LogicalAnd => self.add_lexeme(LexemeData::LogicalAnd, 1),
                TokenKind::LogicalOr => self.add_lexeme(LexemeData::LogicalOr, 1),
                TokenKind::NotEquals => self.add_lexeme(LexemeData::NotEquals, 1),
                TokenKind::Underscore => self.add_lexeme(LexemeData::Underscore, 1),
                TokenKind::Colon => self.add_lexeme(LexemeData::TypeAnnSeparator, 1),
                TokenKind::DoubleColon => self.add_lexeme(LexemeData::PathSeparator, 1),
                TokenKind::Open(enclosure) => self.add_level(enclosure),
                TokenKind::Close(enclosure) => return self.close(t, enclosure),
                TokenKind::Digits => self.add_digits(t),
                TokenKind::Word => self.add_word(t),
                _ => panic!("Unexpected token"),
            }
        }
        self.output()
    }

    fn add_lexeme(&mut self, data: LexemeData, tokens: usize) {
        debug_assert!(tokens > 0);
        let from = &self.tokens.get(self.index).unwrap();
        let range = if tokens == 1 {
            from.range.clone()
        } else {
            from.range
                .merge(&self.tokens.get(self.index + tokens - 1).unwrap().range)
                .unwrap()
        };
        self.output.add_value(Lexeme {
            position: Position::String(range),
            separator: self.separator,
            data,
        });
        self.advance(tokens);
        self.separator = Separator::Nothing;
    }

    fn advance_whitespace(&mut self, separator: Separator) {
        self.advance(1);
        self.separator = self.separator + separator;
    }

    fn add_level(&mut self, enclosure: Enclosure) {
        let start_index = self.index + 1;
        let (result, end_index) = Lexer {
            separator: Separator::Nothing,
            tokens: self.tokens,
            output: Output::empty(),
            index: start_index,
            lexeme_start: start_index,
            enclosed_by: Some(enclosure),
        }
        .lex();
        // Base case "()":
        // - self.index = 0
        // - start_index = 1
        // - end_index = 1
        // - Tokens to consume = 1 + (end_index - start_index) + 1
        let ntokens = 2 + end_index - start_index;
        self.output.merge_problems(result).map(|lexemes| {
            self.add_lexeme(LexemeData::Level(Level { enclosure, lexemes }), ntokens)
        });
        /*
        match self.problems.add_result(result) {
            Some(lexemes) => {
                self.add_lexeme(LexemeData::Level(Level { enclosure, lexemes }), ntokens);
            }
            _ => panic!("TODO"),
        } */
    }

    fn close(mut self, token: Token, enclosure: Enclosure) -> (Output<Lexemes>, usize) {
        match self.enclosed_by {
            None => self.add_error(&token, Error::NoOpenEnclosure, 0),
            Some(e) => {
                if enclosure != e {
                    self.add_error(&token, Error::InvalidOpenEnclosure, 1);
                    // Advance until the right close token
                    let mut n = 0;
                    while let Some(t) = self.peek_ahead(n) {
                        if TokenKind::Close(enclosure) == t.kind {
                            break;
                        } else {
                            n += 1;
                        }
                    }
                }
            }
        }
        self.output()
    }

    fn add_digits(&mut self, digits: Token) {
        self.add_lexeme(
            LexemeData::Integer(digits.range.substr().parse().unwrap()),
            1,
        );
    }

    fn add_word(&mut self, token: Token) {
        let word = token.range.substr();
        if let Some(k) = Keyword::parse(word.as_str()) {
            self.add_lexeme(LexemeData::Keyword(k), 1);
        } else {
            let name = word.as_str();
            if let Ok(s) = TSymbol::new(name) {
                self.add_lexeme(LexemeData::TSymbol(s), 1);
            } else if let Ok(s) = Symbol::new(name) {
                self.add_lexeme(LexemeData::Symbol(s), 1);
            } else {
                self.add_error(&token, Error::InvalidWord, 1)
            }
        }
    }

    fn add_error(&mut self, token: &Token, error: Error, tokens: usize) {
        self.output
            .add_error(error.at(Position::String(token.range.clone())));
        self.advance(tokens)
    }
}

#[cfg(test)]
mod tests;
