pub mod scanner;

use arendal_error::error;
use arendal_error::errors::Error;
use arendal_error::Error as E;

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

#[derive(Debug)]
struct ParserError {
    line: usize,
    index: usize,
    error_type: ErrorType,
}

#[derive(Debug)]
enum ErrorType {
    IndentationError,
    UnexpectedChar(char),
}

impl Error for ParserError {}

fn indentation_error(line: usize, index: usize) -> E {
    error(ParserError {
        line,
        index,
        error_type: ErrorType::IndentationError,
    })
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
