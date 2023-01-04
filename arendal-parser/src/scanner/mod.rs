mod errors;

use arendal_error::{ErrorCollector, Result};
use errors::*;

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

    fn is_done(&self) -> bool {
        self.char_index >= self.chars.len()
    }

    fn consume(&mut self) {
        self.index += self.chars[self.char_index].0;
        self.char_index += 1;
    }

    fn scan(mut self) -> Result<Vec<Token>> {
        while !self.is_done() {
            self.line = self.line + 1;
            let (indentation, ok) = super::Indentation::get(&self.input[self.index..]);
            if !ok {
                self.errors.add(indentation_error())
            }
            let len = indentation.len();
            if len > 0 {
                self.index = self.index + len;
                self.add_indentation(indentation);
            }
            break; // TODO: next step skip whitespace and start looking at characters
        }
        self.errors.to_result(self.tokens)
    }

    fn add_indentation(&mut self, indentation: Indentation) {
        self.tokens.push(Token {
            line: self.line,
            index: 0,
            token_type: TokenType::Indentation(indentation),
        });
    }
}
