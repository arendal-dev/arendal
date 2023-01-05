pub mod pos;
pub mod tokenizer1; // Tokenizer - first pass
pub mod tokenizer2; // Tokenizer - second pass

use arendal_ast::error::{Error, Errors, Result};
pub use pos::Pos;

#[derive(Debug, Clone, PartialEq, Eq)]
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

    fn chars(self) -> usize {
        self.bytes() // we have another method in case it's different in the future
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enclosure {
    Parens,
    Square,
    Curly,
}

#[derive(Debug)]
struct ParserError<'a> {
    // Error position
    pos: Pos<'a>,
    error_type: ErrorType<'a>,
}

impl<'a> ParserError<'a> {
    fn new(pos: Pos<'a>, error_type: ErrorType<'a>) -> Self {
        ParserError { pos, error_type }
    }
}

#[derive(Debug)]
enum ErrorType<'a> {
    IndentationError,
    UnexpectedChar(char),
    UnexpectedToken(tokenizer1::Token<'a>),
}

impl<'a> Error for ParserError<'a> {}

fn indentation_error(pos: Pos) -> ParserError {
    ParserError::new(pos, ErrorType::IndentationError)
}

fn unexpected_char(pos: Pos, c: char) -> ParserError {
    ParserError::new(pos, ErrorType::UnexpectedChar(c))
}

fn unexpected_token(token: tokenizer1::Token) -> ParserError {
    ParserError::new(token.pos, ErrorType::UnexpectedToken(token))
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
