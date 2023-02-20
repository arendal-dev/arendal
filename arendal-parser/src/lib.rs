pub mod lexer;
pub mod parser;
pub mod tokenizer;

use std::cmp::{Ord, Ordering, PartialOrd};

use lexer::{Lexeme, LexemeKind, LexemeRef, Lexemes, Line, Lines};
use tokenizer::{Token, TokenKind, Tokens};

pub use parser::parse_expression;

use ast::error::{Error, Errors, Result};
use ast::{ArcStr, Expression, Loc, Substr};
use num::Integer;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enclosure {
    Parens,
    Square,
    Curly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewLine {
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

// This struct represents an input string and a byte index in it.
#[derive(Clone, PartialEq, Eq)]
struct Pos {
    input: ArcStr, // Input string
    index: usize,  // Byte index from the beginning of the input
}

impl Pos {
    // Creates a new position at the beginning of the input
    fn new(input: ArcStr) -> Pos {
        Pos { input, index: 0 }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.index >= self.input.len()
    }

    // Advances the current position the provided number of bytes
    fn advance(&mut self, bytes: usize) {
        self.index += bytes;
    }

    // Advances the current position the length of the provided char
    fn advance_char(&mut self, c: char) {
        self.advance(c.len_utf8());
    }

    // Resets the current position
    fn reset(&mut self) {
        self.index = 0;
    }

    // Returns a slice from the current position to provided one
    // Panics if the positions are for different input or if the end index is smaller
    // than the current one or larger than the length of the input.
    fn str_to(&self, to: &Pos) -> Substr {
        assert_eq!(self.input, to.input);
        assert!(self.index <= to.index);
        self.input.substr(self.index..to.index)
    }
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pos({})", self.index)
    }
}

impl From<Pos> for Loc {
    fn from(value: Pos) -> Self {
        Loc::input(value.input.clone(), value.index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Indentation {
    tabs: usize,
    spaces: usize,
}

impl Indentation {
    #[inline]
    fn new(tabs: usize, spaces: usize) -> Indentation {
        Indentation { tabs, spaces }
    }

    fn next(&self) -> Indentation {
        Self::new(self.tabs, self.spaces + 1)
    }
}

impl Ord for Indentation {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self.tabs.cmp(&other.tabs);
        if ordering == Ordering::Equal {
            self.spaces.cmp(&other.spaces)
        } else {
            ordering
        }
    }
}

impl PartialOrd for Indentation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::Pos;
    use ast::literal;

    #[test]
    fn pos_harness() {
        let mut pos = Pos::new(literal!("1234"));
        assert_eq!(0, pos.index);
        assert!(!pos.is_done());
        pos.advance(2);
        assert_eq!(2, pos.index);
        assert!(!pos.is_done());
        pos.advance(3);
        assert_eq!(5, pos.index);
        assert!(pos.is_done());
        pos.reset();
        assert_eq!(0, pos.index);
        assert!(!pos.is_done());
    }
}
