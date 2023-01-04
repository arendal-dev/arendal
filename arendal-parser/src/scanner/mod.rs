mod errors;

use arendal_error::{ErrorCollector, Result};
use errors::*;

use crate::Indentation;

fn scan(input: &str) -> Result<Vec<Token>> {
    Scanner::new(input).scan()
}

#[derive(Debug)]
struct Token {
    line: usize,
    index: usize, // relative to the line, not the input.
    tokenType: TokenType,
}

#[derive(Debug)]
enum TokenType {
    Indentation(super::Indentation),
}

impl Token {
    fn indentation(line: usize, indentation: Indentation) -> Token {
        Token {
            line,
            index: 0,
            tokenType: TokenType::Indentation(indentation),
        }
    }
}

struct Scanner<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>, 
    tokens : Vec<Token>,
    errors : ErrorCollector,
    line : usize, // Current line
    index: usize, // Current index from the beginning of the input
    charIndex: usize, // Current char index from the beginning of the input
    lineIndex: usize, // Current index from the beginning of the line
    lexeme: usize, // Indext of the start of the current lexeme
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
            charIndex: 0,
            lineIndex: 0,
            lexeme: 0,
        }
    }

    fn is_done(&self) -> bool {
        self.charIndex >= self.chars.len()
    }

    fn consume(&mut self) {
        self.index += self.chars[self.charIndex].0;
        self.charIndex += 1;
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
                self.tokens.push(Token::indentation(self.line, indentation))
            }
            break; // TODO: next step skip whitespace and start looking at characters
        }
        self.errors.to_result(self.tokens)
    }

}

