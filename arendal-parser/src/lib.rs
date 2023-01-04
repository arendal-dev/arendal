pub mod tokenizer1; // Tokenizer - first pass
pub mod tokenizer2; // Tokenizer - second pass

use arendal_error::{Error, Errors};

#[derive(Debug, PartialEq, Eq)]
struct Indentation {
    tabs: usize,
    spaces: usize,
}

impl Indentation {
    #[inline]
    fn new(tabs: usize, spaces: usize) -> Indentation {
        Indentation { tabs, spaces }
    }

    fn get(input: &str) -> (Indentation, bool) {
        let mut tabs: usize = 0;
        let mut spaces: usize = 0;
        let mut ok: bool = true;
        for (_, c) in input.char_indices() {
            if c == '\t' {
                if spaces > 0 {
                    ok = false;
                    break;
                }
                tabs += 1;
            } else if c == ' ' {
                spaces += 1;
            } else {
                break;
            }
        }
        (Self::new(tabs, spaces), ok)
    }

    #[inline]
    fn len(&self) -> usize {
        self.tabs + self.spaces
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewLine {
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
}

// The coordinates of a token or an error in the input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    // Line number, starts at 1
    line: usize,
    // Byte index from the beginning of the input
    index: usize,
    // Byte index from the beginning of the line
    line_index: usize,
}

impl Pos {
    // Creates a new position at the beginning of the input
    fn new() -> Pos {
        Pos {
            line: 1,
            index: 0,
            line_index: 0,
        }
    }

    // Returns a new position that it's the provided number of bytes forward in the same line
    fn advance(&self, bytes: usize) -> Pos {
        Pos {
            line: self.line,
            index: self.index + bytes,
            line_index: self.line_index + bytes,
        }
    }

    // Returns a new position at the beginning of the next line.
    fn newline(&self, newline: NewLine) -> Pos {
        Pos {
            line: self.line + 1,
            index: self.index + newline.bytes(),
            line_index: 0,
        }
    }
}

#[derive(Debug)]
struct ParserError {
    // Error position
    pos: Pos,
    error_type: ErrorType,
}

impl ParserError {
    fn new(pos: Pos, error_type: ErrorType) -> Self {
        ParserError { pos, error_type }
    }
}

#[derive(Debug)]
enum ErrorType {
    IndentationError,
    UnexpectedChar(char),
}

impl Error for ParserError {}

fn indentation_error(pos: Pos) -> ParserError {
    ParserError::new(pos, ErrorType::IndentationError)
}

fn unexpected_char(pos: Pos, c: char) -> ParserError {
    ParserError::new(pos, ErrorType::UnexpectedChar(c))
}

#[cfg(test)]
mod tests {

    use super::Indentation;

    fn test_indentation(input: &str, tabs: usize, spaces: usize, ok: bool) {
        assert_eq!(
            Indentation::get(input),
            (Indentation::new(tabs, spaces), ok)
        );
    }

    #[test]
    fn indentation() {
        test_indentation("1", 0, 0, true);
        test_indentation("\t1", 1, 0, true);
        test_indentation("\t\t1", 2, 0, true);
        test_indentation("\t\t 1", 2, 1, true);
        test_indentation("\t\t  1", 2, 2, true);
    }
}
