pub mod pass1;

use arendal_ast::error;
use std::fmt;

// This struct represents an input string and a byte index in it.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Pos<'a> {
    input: &'a str, // Input string
    index: usize,   // Byte index from the beginning of the input
}

impl<'a> Pos<'a> {
    // Creates a new position at the beginning of the input
    fn new(input: &str) -> Pos {
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

    // Resets the current position
    fn reset(&mut self) {
        self.index = 0;
    }

    // Returns a slice from the current position to provided one
    // Panics if the positions are for different input or if the end index is smaller
    // than the current one or larger than the length of the input.
    fn str_to(&self, to: &Pos) -> &'a str {
        assert_eq!(self.input, to.input);
        assert!(self.index <= to.index);
        &self.input[self.index..to.index]
    }
}

impl<'a> fmt::Debug for Pos<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pos({})", self.index)
    }
}

impl<'a> error::ErrorLoc for Pos<'a> {}

type Errors<'a> = error::Errors<'a, Pos<'a>>;
type Result<'a, T> = error::Result<'a, T, Pos<'a>>;
type Expression<'a> = arendal_ast::Expression<'a, Pos<'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indentation {
    tabs: usize,
    spaces: usize,
}

impl Indentation {
    #[inline]
    fn new(tabs: usize, spaces: usize) -> Indentation {
        Indentation { tabs, spaces }
    }
}

#[derive(Debug)]
struct ParserError<'a> {
    // Error position
    pos: Pos<'a>,
    error_type: ErrorType,
}

impl<'a> ParserError<'a> {
    fn new(pos: Pos<'a>, error_type: ErrorType) -> Self {
        ParserError { pos, error_type }
    }
}

#[derive(Debug)]
enum ErrorType {
    IndentationError,
    UnexpectedChar(char),
    ParsingError, // placeholder, temporary error
}

impl<'a> error::Error<'a, Pos<'a>> for ParserError<'a> {
    fn location(&self) -> Pos<'a> {
        self.pos
    }
}

fn indentation_error(pos: Pos) -> ParserError {
    ParserError::new(pos, ErrorType::IndentationError)
}

fn unexpected_char(pos: Pos, c: char) -> ParserError {
    ParserError::new(pos, ErrorType::UnexpectedChar(c))
}

fn parsing_error(pos: Pos) -> ParserError {
    ParserError::new(pos, ErrorType::ParsingError)
}

#[cfg(test)]
mod tests {
    use super::Pos;

    #[test]
    fn pos_harness() {
        let mut pos = Pos::new("1234");
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
