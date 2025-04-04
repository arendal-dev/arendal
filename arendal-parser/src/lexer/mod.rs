mod tokenizer;

use std::fmt;
use std::rc::Rc;

use arcstr::Substr;
use ast::position::Position;
use ast::problem::{Problems, Result};
use ast::keyword::Keyword;
use ast::symbol::{Symbol, TSymbol};
use num::Integer;
use ast::input::StringInput;
use tokenizer::{tokenize, Token, TokenData, Tokens};

pub(super) fn lex(input: StringInput) -> Result<Lexemes> {
    let output = tokenize(input)?;
    Lexer::new(output).lex().0
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

impl Separator {
    fn add(self, other: Separator) -> Self {
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

#[derive(Clone, Eq, PartialEq)]
pub(super) struct Lexeme {
    pub(super) position: Position,
    pub(super) separator: Separator,
    pub(super) data: LexemeData,
}

impl fmt::Debug for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}]{:?}[{:?}]", self.position, self.separator, self.data)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub(super) struct Lexemes {
    lexemes: Rc<Vec<Lexeme>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Level {
    pub(super) enclosure: Enclosure,
    pub(super) lexemes: Lexemes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    PathSeparator,
    Integer(Integer),
    Level(Level),
    Underscore,
    Symbol(Symbol),
    TSymbol(TSymbol),
    Keyword(Keyword),
}

struct Lexer {
    separator: Separator,
    tokens: Tokens,
    lexemes: Vec<Lexeme>,
    problems: Problems,
    index: usize,        // Index of the current input token
    lexeme_start: usize, // Index of the start token of the current lexeme
    enclosed_by: Option<Enclosure>,
}

impl Lexer {
    fn new(input: (Tokens, Problems)) -> Lexer {
        let (tokens, problems) = input;
        Lexer {
            separator: Separator::Start,
            tokens,
            lexemes: Vec::default(),
            problems,
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

    fn output(self) -> (Result<Lexemes>, usize) {
        (self.problems.to_lazy_result(|| Lexemes { lexemes: Rc::new(self.lexemes) }), self.index)
    }

    fn lex(mut self) -> (Result<Lexemes>, usize) {
        while let Some(t) = self.peek().cloned() {
            self.lexeme_start = self.index;
            match t.data {
                TokenData::Tabs(_) | TokenData::Spaces(_) => self.advance_whitespace(Separator::Whitespace),
                TokenData::EndOfLine(_) => self.advance_whitespace(Separator::NewLine),
                TokenData::Plus => self.add_lexeme(LexemeData::Plus, 1),
                TokenData::Minus => self.add_lexeme(LexemeData::Minus, 1),
                TokenData::Star => self.add_lexeme(LexemeData::Star, 1),
                TokenData::Slash => self.add_lexeme(LexemeData::Slash, 1),
                TokenData::Dot => self.add_lexeme(LexemeData::Dot, 1),
                TokenData::Greater => self.add_lexeme(LexemeData::Greater, 1),
                TokenData::GreaterOrEq => self.add_lexeme(LexemeData::GreaterOrEq, 1),
                TokenData::Less => self.add_lexeme(LexemeData::Less, 1),
                TokenData::LessOrEq => self.add_lexeme(LexemeData::LessOrEq, 1),
                TokenData::Bang => self.add_lexeme(LexemeData::Bang, 1),
                TokenData::Assignment => self.add_lexeme(LexemeData::Assignment, 1),
                TokenData::Equals => self.add_lexeme(LexemeData::Equals, 1),
                TokenData::LogicalAnd => self.add_lexeme(LexemeData::LogicalAnd, 1),
                TokenData::LogicalOr => self.add_lexeme(LexemeData::LogicalOr, 1),
                TokenData::NotEquals => self.add_lexeme(LexemeData::NotEquals, 1),
                TokenData::Underscore => self.add_lexeme(LexemeData::Underscore, 1),
                TokenData::DoubleColon => self.add_lexeme(LexemeData::PathSeparator, 1),
                TokenData::Open(enclosure) => self.add_level(enclosure),
                TokenData::Close(enclosure) => return self.close(t.position, enclosure),
                TokenData::Digits(s) => self.add_digits(&s),
                TokenData::Word(s) => self.add_word(t.position, &s),
                _ => panic!("Unexpected token"),
            }
        }
        self.output()
    }

    fn add_lexeme(&mut self, data: LexemeData, tokens: usize) {
        debug_assert!(tokens > 0);
        let from = &self.tokens.get(self.index).unwrap();
        let position = if tokens == 1 {
            from.position.clone()
        } else {
            from.position.merge(&self.tokens.get(self.index+tokens-1).unwrap().position)
        };
        self.lexemes.push(Lexeme {
            position,
            separator: self.separator,
            data
        });
        self.advance(tokens);
        self.separator = Separator::Nothing;
    }

    fn advance_whitespace(&mut self, separator: Separator) {
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
        self.separator.add(separator);
    }

    fn add_level(&mut self, enclosure: Enclosure) {
        let start_index = self.index + 1;
        let (result, end_index) = Lexer {
            separator: self.separator,
            tokens: self.tokens.clone(),
            lexemes: Vec::default(),
            problems: Problems::default(),
            index: start_index,
            lexeme_start: start_index,
            enclosed_by: Some(enclosure),
        }.lex();
        let ntokens = 1 + end_index - start_index;
        match result {
            Ok((lexemes, problems)) => {
                self.problems.add_problems(problems);
                self.add_lexeme(LexemeData::Level(Level { enclosure, lexemes }), ntokens);
            },
            _ => panic!("TODO")
        }
    }

    fn close(mut self, position: Position, enclosure: Enclosure) -> (Result<Lexemes>, usize) {
        match self.enclosed_by {
            None => self.add_error(position, "TODO", format!("No open enclosure"), 0),
            Some(e) => if enclosure != e {
                self.add_error(position, "TODO", format!("Invalid close enclosure"), 1);
                // Advance until the right close token
                let mut n = 0;
                while let Some(t) = self.peek_ahead(n) {
                    if TokenData::Close(enclosure) == t.data {
                        break;
                    } else {
                        n += 1;
                    }
                }
            }
        }
        self.output()
    }

/*
    fn add_close(&mut self, loc: &Loc, e: Enclosure) {
        match self.enclosures.pop() {
            Some(last) => {
                if e == last {
                    self.add_lexeme(LexemeData::Close(e), 1);
                    return;
                }
                // Advance until the right close token
                let mut n = 1;
                while let Some(t) = self.peek_ahead(n) {
                    if TokenData::Close(e) == t.it {
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
*/
    fn add_digits(&mut self, digits: &Substr) {
        self.add_lexeme(LexemeData::Integer(digits.parse().unwrap()), 1);
    }

    fn add_word(&mut self, position: Position, word: &Substr) {
        if let Some(k) = Keyword::parse(word) {
            self.add_lexeme(LexemeData::Keyword(k), 1);
        } else {
            let name = word.as_str();
            if let Ok(s) = TSymbol::new(name) {
                self.add_lexeme(LexemeData::TSymbol(s), 1);
            } else if let Ok(s) = Symbol::new(name) {
                self.add_lexeme(LexemeData::Symbol(s), 1);
            } else {
                self.add_error(position, "TODO", format!("Invalid word"), 1)
            }
        }
    }

    fn add_error(&mut self, position: Position, code: &str, message: String, tokens: usize) {
        self.problems.add_error(position.clone(), code, message);
        self.advance(tokens)
    }
}

#[cfg(test)]
mod tests;
