use arendal_error::{ErrorCollector, Result};

use crate::Indentation;

pub fn scan(input: &str) -> Result<Vec<Token>> {
    Scanner::new(input).scan()
}

#[derive(Debug)]
pub struct Token {
    line: usize,
    index: usize, // relative to the line, not the input.
    token_type: TokenType,
}

#[derive(Debug)]
enum TokenType {
    Indentation(super::Indentation),
}

impl Token {}

struct Scanner<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    tokens: Vec<Token>,
    errors: ErrorCollector,
    line: usize,        // Current line
    index: usize,       // Current index from the beginning of the input
    char_index: usize,  // Current char index from the beginning of the input
    line_index: usize,  // Current index from the beginning of the line
    token_start: usize, // Index of the start of the current token
}

impl<'a> Scanner<'a> {
    fn new(input: &str) -> Scanner {
        Scanner {
            input,
            chars: input.char_indices().collect(),
            tokens: Vec::new(),
            errors: ErrorCollector::new(),
            line: 0,
            index: 0,
            char_index: 0,
            line_index: 0,
            token_start: 0,
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.char_index >= self.chars.len()
    }

    // Advances the indexes one char
    fn advance(&mut self) {
        self.index += self.chars[self.char_index].0;
        self.char_index += 1;
    }

    // Returns the char at the current index.
    // Panics if we have reached the end of the input
    fn peek(&self) -> char {
        self.chars[self.char_index].1
    }

    // Returns the char at the following index, if any.
    fn peek_next(&self) -> Option<char> {
        let i = self.char_index + 1;
        if i >= self.chars.len() {
            None
        } else {
            Some(self.chars[i].1)
        }
    }

    // Advances one char if we are not at the end of the input
    // and the current char is the same as the provided.
    fn advance_if_char(&mut self, c: char) -> bool {
        if !self.is_done() && c == self.peek() {
            self.advance();
            true
        } else {
            false
        }
    }

    // Advances two chars if we are at least two chars from the end of the input
    // and the current and next ones are same as the provided.
    fn advance_if_two_char(&mut self, c1: char, c2: char) -> bool {
        let i = self.char_index + 1;
        if i >= self.chars.len() {
            if c1 == self.peek() && c2 == self.chars[i].1 {
                self.advance();
                self.advance();
                return true;
            }
        }
        false
    }

    fn skip_whitespace(&mut self) {
        while self.advance_if_char(' ') || self.advance_if_char('\t') {}
    }

    // Returns true if we are reached the end of the input or found
    // the end of a line. In the latter case, the chars are advanced.
    fn is_done_or_eol(&mut self) -> bool {
        self.is_done() || self.advance_if_char('\n') || self.advance_if_two_char('\r', '\n')
    }

    fn scan(mut self) -> Result<Vec<Token>> {
        while !self.is_done() {
            self.line += 1;
            let (indentation, ok) = super::Indentation::get(&self.input[self.index..]);
            let len = indentation.len();
            if len > 0 {
                self.index += len;
                self.add_indentation(indentation);
            }
            if !ok {
                self.add_indentation_error();
            }
            self.scan_line();
        }
        self.errors.to_result(self.tokens)
    }

    fn scan_line(&mut self) {
        while !self.is_done_or_eol() {
            self.skip_whitespace();
            self.add_unexpected_char(self.peek());
        }
    }

    fn add_indentation(&mut self, indentation: Indentation) {
        self.tokens.push(Token {
            line: self.line,
            index: 0,
            token_type: TokenType::Indentation(indentation),
        });
    }

    fn add_indentation_error(&mut self) {
        self.errors
            .add(super::indentation_error(self.line, self.line_index))
    }

    fn add_unexpected_char(&mut self, c: char) {
        self.errors
            .add(super::unexpected_char(self.line, self.line_index, c))
    }
}
